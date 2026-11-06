"""Consumer Python của hợp đồng LedgerEntryPosted: đọc fixture do Rust sinh bằng thư viện protobuf.

Schema lấy từ descriptor set (cùng tệp mọi consumer dùng), không sinh code Python.
Dùng: python consume.py <thư-mục-fixtures>
In mỗi ca một dòng JSON: {"id":..,"ok":true,"view":{..}} hoặc {"id":..,"ok":false,"error":".."}.
"""
import json
import os
import sys

from google.protobuf import descriptor_pb2, descriptor_pool, json_format, message_factory


def load_pool(path):
    fds = descriptor_pb2.FileDescriptorSet()
    with open(path, "rb") as f:
        fds.ParseFromString(f.read())
    pool = descriptor_pool.DescriptorPool()
    for file_proto in fds.file:  # protoc đã sắp theo thứ tự phụ thuộc
        pool.Add(file_proto)
    return pool


def message_name(reader):
    if reader == "s1":
        return "toinoi.events.v1.LedgerEntryPosted"
    return "toinoi.lab.%s.LedgerEntryPosted" % reader


def decode(pool, directory, case):
    desc = pool.FindMessageTypeByName(message_name(case["reader"]))
    cls = message_factory.GetMessageClass(desc)
    msg = cls()
    with open(os.path.join(directory, case["file"]), "rb") as f:
        data = f.read()
    try:
        if case["format"] == "bin":
            msg.ParseFromString(data)
        elif case["format"] == "json":
            json_format.Parse(data.decode("utf-8"), msg)
        else:
            json_format.Parse(data.decode("utf-8"), msg, ignore_unknown_fields=True)
    except Exception as exc:  # noqa: BLE001 - báo nguyên văn lỗi của thư viện
        return {"ok": False, "error": "%s: %s" % (type(exc).__name__, exc)}
    view = json_format.MessageToDict(msg, always_print_fields_with_no_presence=True)
    return {"ok": True, "view": view}


def lookup(value, path):
    for part in path.split("."):
        value = value[int(part)] if part.isdigit() else value[part]
    return value


def naive(directory, case):
    with open(os.path.join(directory, case["file"]), "rb") as f:
        generic = json.loads(f.read())
    value = lookup(generic, case["path"])
    out = {"default": "%s %s" % (type(value).__name__, value)}
    # Python: số nguyên JSON là int chính xác tùy ý; chỉ mất khi chủ động ép float.
    out["viaDouble"] = "%.0f" % float(value) if isinstance(value, (int, float)) else None
    out["viaInt64"] = str(int(value))
    return {"ok": True, "value": out}


def main():
    directory = sys.argv[1]
    pool = load_pool(os.path.join(directory, "schemas.binpb"))
    with open(os.path.join(directory, "cases.json"), "rb") as f:
        cases = json.loads(f.read())
    for case in cases:
        body = decode(pool, directory, case) if case["kind"] == "decode" else naive(directory, case)
        body["id"] = case["id"]
        print(json.dumps(body, ensure_ascii=False))


if __name__ == "__main__":
    main()
