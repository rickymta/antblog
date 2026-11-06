{{- define "toinoi-service.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "toinoi-service.fullname" -}}
{{- printf "toinoi-%s-%s" .Values.env (include "toinoi-service.name" .) | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "toinoi-service.labels" -}}
app.kubernetes.io/name: {{ include "toinoi-service.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
Project: ToiNoi
Env: {{ .Values.env }}
Service: {{ include "toinoi-service.name" . }}
{{- end -}}

{{- define "toinoi-service.selectorLabels" -}}
app.kubernetes.io/name: {{ include "toinoi-service.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "toinoi-service.serviceAccountName" -}}
{{- if .Values.serviceAccount.name -}}
{{- .Values.serviceAccount.name -}}
{{- else -}}
{{- include "toinoi-service.name" . -}}
{{- end -}}
{{- end -}}

{{- define "toinoi-service.image" -}}
{{- if .Values.image.digest -}}
{{- printf "%s@%s" .Values.image.repository .Values.image.digest -}}
{{- else -}}
{{- fail "image.digest là bắt buộc: ghim image theo digest sha256" -}}
{{- end -}}
{{- end -}}
