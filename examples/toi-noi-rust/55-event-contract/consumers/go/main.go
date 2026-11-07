// Consumer Go của hợp đồng LedgerEntryPosted: đọc fixture do Rust sinh bằng protobuf-go + protojson.
// Schema lấy từ descriptor set (cùng tệp mọi consumer dùng), không sinh code Go.
//
// Dùng: go run . <thư-mục-fixtures>
// In ra mỗi ca một dòng JSON: {"id":..,"ok":true,"view":{..}} hoặc {"id":..,"ok":false,"error":".."}.
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protodesc"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/descriptorpb"
	"google.golang.org/protobuf/types/dynamicpb"
)

type caseSpec struct {
	ID     string `json:"id"`
	Kind   string `json:"kind"`
	Reader string `json:"reader"`
	Format string `json:"format"`
	File   string `json:"file"`
	Path   string `json:"path"`
}

func main() {
	if len(os.Args) != 2 {
		fmt.Fprintln(os.Stderr, "dùng: go run . <thư-mục-fixtures>")
		os.Exit(2)
	}
	dir := os.Args[1]

	setBytes := must(os.ReadFile(filepath.Join(dir, "schemas.binpb")))
	var set descriptorpb.FileDescriptorSet
	check(proto.Unmarshal(setBytes, &set))
	files := must(protodesc.NewFiles(&set))

	var cases []caseSpec
	check(json.Unmarshal(must(os.ReadFile(filepath.Join(dir, "cases.json"))), &cases))

	for _, c := range cases {
		switch c.Kind {
		case "decode":
			emit(c.ID, decode(files, dir, c))
		case "naive":
			emit(c.ID, naive(dir, c))
		}
	}
}

func messageName(reader string) protoreflect.FullName {
	if reader == "s1" {
		return "toinoi.events.v1.LedgerEntryPosted"
	}
	return protoreflect.FullName("toinoi.lab." + reader + ".LedgerEntryPosted")
}

// decode đọc một fixture bằng schema của reader; lỗi của thư viện trả về nguyên văn.
func decode(files interface {
	FindDescriptorByName(protoreflect.FullName) (protoreflect.Descriptor, error)
}, dir string, c caseSpec) map[string]any {
	d, err := files.FindDescriptorByName(messageName(c.Reader))
	if err != nil {
		return fail(err)
	}
	md := d.(protoreflect.MessageDescriptor)
	data := must(os.ReadFile(filepath.Join(dir, c.File)))
	msg := dynamicpb.NewMessage(md)
	switch c.Format {
	case "bin":
		err = proto.Unmarshal(data, msg)
	case "json":
		err = protojson.UnmarshalOptions{}.Unmarshal(data, msg)
	case "json-lenient":
		err = protojson.UnmarshalOptions{DiscardUnknown: true}.Unmarshal(data, msg)
	}
	if err != nil {
		return fail(err)
	}
	// View của reader: proto3 JSON chuẩn, đủ cả trường mặc định.
	out, err := protojson.MarshalOptions{EmitUnpopulated: true}.Marshal(msg)
	if err != nil {
		return fail(err)
	}
	return map[string]any{"ok": true, "view": json.RawMessage(out)}
}

// naive đọc JSON thuần như client hay làm: `any` cho số float64 (mặc định của encoding/json).
func naive(dir string, c caseSpec) map[string]any {
	data := must(os.ReadFile(filepath.Join(dir, c.File)))
	var generic any
	check(json.Unmarshal(data, &generic))
	def := lookup(generic, c.Path)

	dec := json.NewDecoder(bytes.NewReader(data))
	dec.UseNumber()
	var exact any
	check(dec.Decode(&exact))
	num := lookup(exact, c.Path)

	value := map[string]any{"default": fmt.Sprintf("%T %v", def, formatAny(def))}
	switch v := def.(type) {
	case float64:
		value["viaDouble"] = strconv.FormatFloat(v, 'f', 0, 64)
	default:
		value["viaDouble"] = nil
	}
	switch n := num.(type) {
	case json.Number:
		value["viaInt64"] = strconv.FormatInt(must(n.Int64()), 10)
	case string:
		value["viaInt64"] = strconv.FormatInt(must(strconv.ParseInt(n, 10, 64)), 10)
	}
	return map[string]any{"ok": true, "value": value}
}

func formatAny(v any) string {
	if f, ok := v.(float64); ok {
		return strconv.FormatFloat(f, 'f', 0, 64)
	}
	return fmt.Sprint(v)
}

func lookup(v any, path string) any {
	for _, part := range strings.Split(path, ".") {
		if idx, err := strconv.Atoi(part); err == nil {
			v = v.([]any)[idx]
		} else {
			v = v.(map[string]any)[part]
		}
	}
	return v
}

func fail(err error) map[string]any {
	return map[string]any{"ok": false, "error": err.Error()}
}

func emit(id string, body map[string]any) {
	body["id"] = id
	line, err := json.Marshal(body)
	check(err)
	fmt.Println(string(line))
}

func must[T any](v T, err error) T {
	check(err)
	return v
}

func check(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, "lỗi:", err)
		os.Exit(1)
	}
}
