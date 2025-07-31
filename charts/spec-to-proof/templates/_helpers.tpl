{{/*
Expand the name of the chart.
*/}}
{{- define "spec-to-proof.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "spec-to-proof.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "spec-to-proof.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "spec-to-proof.labels" -}}
helm.sh/chart: {{ include "spec-to-proof.chart" . }}
{{ include "spec-to-proof.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "spec-to-proof.selectorLabels" -}}
app.kubernetes.io/name: {{ include "spec-to-proof.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "spec-to-proof.serviceAccountName" -}}
{{- if .Values.serviceAccounts.create }}
{{- default (include "spec-to-proof.fullname" .) .Values.serviceAccounts.name }}
{{- else }}
{{- default "default" .Values.serviceAccounts.name }}
{{- end }}
{{- end }}

{{/*
Create the name of the config map to use
*/}}
{{- define "spec-to-proof.configMapName" -}}
{{- printf "%s-config" (include "spec-to-proof.fullname" .) }}
{{- end }}

{{/*
Create the name of the secret to use
*/}}
{{- define "spec-to-proof.secretName" -}}
{{- printf "%s-secret" (include "spec-to-proof.fullname" .) }}
{{- end }}

{{/*
Create the name of the service to use
*/}}
{{- define "spec-to-proof.serviceName" -}}
{{- printf "%s-service" (include "spec-to-proof.fullname" .) }}
{{- end }}

{{/*
Create the name of the ingress to use
*/}}
{{- define "spec-to-proof.ingressName" -}}
{{- printf "%s-ingress" (include "spec-to-proof.fullname" .) }}
{{- end }}

{{/*
Create the name of the deployment to use
*/}}
{{- define "spec-to-proof.deploymentName" -}}
{{- printf "%s-deployment" (include "spec-to-proof.fullname" .) }}
{{- end }}

{{/*
Create the name of the service account to use for a specific component
*/}}
{{- define "spec-to-proof.componentServiceAccountName" -}}
{{- printf "%s-%s" (include "spec-to-proof.fullname" .) .component }}
{{- end }}

{{/*
Create the name of the config map to use for a specific component
*/}}
{{- define "spec-to-proof.componentConfigMapName" -}}
{{- printf "%s-%s-config" (include "spec-to-proof.fullname" .) .component }}
{{- end }}

{{/*
Create the name of the secret to use for a specific component
*/}}
{{- define "spec-to-proof.componentSecretName" -}}
{{- printf "%s-%s-secret" (include "spec-to-proof.fullname" .) .component }}
{{- end }}

{{/*
Create the name of the service to use for a specific component
*/}}
{{- define "spec-to-proof.componentServiceName" -}}
{{- printf "%s-%s-service" (include "spec-to-proof.fullname" .) .component }}
{{- end }}

{{/*
Create the name of the deployment to use for a specific component
*/}}
{{- define "spec-to-proof.componentDeploymentName" -}}
{{- printf "%s-%s-deployment" (include "spec-to-proof.fullname" .) .component }}
{{- end }}

{{/*
Create the name of the HPA to use for a specific component
*/}}
{{- define "spec-to-proof.componentHPAName" -}}
{{- printf "%s-%s-hpa" (include "spec-to-proof.fullname" .) .component }}
{{- end }}

{{/*
Create the name of the PDB to use for a specific component
*/}}
{{- define "spec-to-proof.componentPDBName" -}}
{{- printf "%s-%s-pdb" (include "spec-to-proof.fullname" .) .component }}
{{- end }}

{{/*
Create the name of the network policy to use for a specific component
*/}}
{{- define "spec-to-proof.componentNetworkPolicyName" -}}
{{- printf "%s-%s-network-policy" (include "spec-to-proof.fullname" .) .component }}
{{- end }}

{{/*
Common labels for a specific component
*/}}
{{- define "spec-to-proof.componentLabels" -}}
{{ include "spec-to-proof.labels" . }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{/*
Selector labels for a specific component
*/}}
{{- define "spec-to-proof.componentSelectorLabels" -}}
{{ include "spec-to-proof.selectorLabels" . }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{/*
Image pull policy
*/}}
{{- define "spec-to-proof.imagePullPolicy" -}}
{{- if .Values.global.imagePullPolicy }}
{{- .Values.global.imagePullPolicy }}
{{- else }}
{{- .Values.images.pullPolicy | default "IfNotPresent" }}
{{- end }}
{{- end }}

{{/*
Image registry
*/}}
{{- define "spec-to-proof.imageRegistry" -}}
{{- if .Values.global.imageRegistry }}
{{- .Values.global.imageRegistry }}
{{- end }}
{{- end }}

{{/*
Image tag
*/}}
{{- define "spec-to-proof.imageTag" -}}
{{- .tag | default "latest" }}
{{- end }}

{{/*
Image name for a specific component
*/}}
{{- define "spec-to-proof.componentImage" -}}
{{- $registry := include "spec-to-proof.imageRegistry" . }}
{{- $image := index .Values.images .component }}
{{- if $registry }}
{{- printf "%s/%s:%s" $registry $image.repository $image.tag }}
{{- else }}
{{- printf "%s:%s" $image.repository $image.tag }}
{{- end }}
{{- end }}

{{/*
Environment variables for a specific component
*/}}
{{- define "spec-to-proof.componentEnv" -}}
- name: POD_NAME
  valueFrom:
    fieldRef:
      fieldPath: metadata.name
- name: POD_NAMESPACE
  valueFrom:
    fieldRef:
      fieldPath: metadata.namespace
- name: NODE_NAME
  valueFrom:
    fieldRef:
      fieldPath: spec.nodeName
- name: SERVICE_NAME
  value: {{ include "spec-to-proof.componentServiceName" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) }}
{{- end }}

{{/*
Security context for a specific component
*/}}
{{- define "spec-to-proof.componentSecurityContext" -}}
{{- $security := index .Values .component "security" }}
{{- if $security }}
securityContext:
  runAsNonRoot: {{ $security.runAsNonRoot | default true }}
  runAsUser: {{ $security.runAsUser | default 1000 }}
  runAsGroup: {{ $security.runAsGroup | default 1000 }}
  fsGroup: {{ $security.fsGroup | default 1000 }}
  allowPrivilegeEscalation: {{ $security.allowPrivilegeEscalation | default false }}
  readOnlyRootFilesystem: {{ $security.readOnlyRootFilesystem | default true }}
  {{- if $security.dropAllCapabilities }}
  capabilities:
    drop:
      - ALL
  {{- end }}
{{- end }}
{{- end }}

{{/*
Container security context for a specific component
*/}}
{{- define "spec-to-proof.componentContainerSecurityContext" -}}
{{- $containerSecurity := index .Values .component "containerSecurity" }}
{{- if $containerSecurity }}
securityContext:
  {{- if $containerSecurity.seccompProfile }}
  seccompProfile:
    type: {{ $containerSecurity.seccompProfile }}
  {{- end }}
  {{- if $containerSecurity.capabilities }}
  capabilities:
    {{- if $containerSecurity.capabilities.drop }}
    drop:
      {{- range $containerSecurity.capabilities.drop }}
      - {{ . }}
      {{- end }}
    {{- end }}
  {{- end }}
{{- end }}
{{- end }}

{{/*
Pod security context for a specific component
*/}}
{{- define "spec-to-proof.componentPodSecurityContext" -}}
{{- $podSecurity := index .Values .component "podSecurityContext" }}
{{- if $podSecurity }}
securityContext:
  runAsNonRoot: {{ $podSecurity.runAsNonRoot | default true }}
  runAsUser: {{ $podSecurity.runAsUser | default 1000 }}
  runAsGroup: {{ $podSecurity.runAsGroup | default 1000 }}
  fsGroup: {{ $podSecurity.fsGroup | default 1000 }}
{{- end }}
{{- end }}

{{/*
Resource requirements for a specific component
*/}}
{{- define "spec-to-proof.componentResources" -}}
{{- $resources := index .Values .component "resources" }}
{{- if $resources }}
resources:
  {{- if $resources.requests }}
  requests:
    {{- if $resources.requests.cpu }}
    cpu: {{ $resources.requests.cpu }}
    {{- end }}
    {{- if $resources.requests.memory }}
    memory: {{ $resources.requests.memory }}
    {{- end }}
  {{- end }}
  {{- if $resources.limits }}
  limits:
    {{- if $resources.limits.cpu }}
    cpu: {{ $resources.limits.cpu }}
    {{- end }}
    {{- if $resources.limits.memory }}
    memory: {{ $resources.limits.memory }}
    {{- end }}
  {{- end }}
{{- end }}
{{- end }}

{{/*
Probe configuration for a specific component
*/}}
{{- define "spec-to-proof.componentProbes" -}}
{{- $probes := index .Values .component "probes" }}
{{- if $probes }}
{{- if $probes.liveness }}
livenessProbe:
  httpGet:
    path: {{ $probes.liveness.path | default "/health" }}
    port: {{ $probes.liveness.port | default "8080" }}
  initialDelaySeconds: {{ $probes.liveness.initialDelaySeconds | default 30 }}
  periodSeconds: {{ $probes.liveness.periodSeconds | default 10 }}
  timeoutSeconds: {{ $probes.liveness.timeoutSeconds | default 5 }}
  failureThreshold: {{ $probes.liveness.failureThreshold | default 3 }}
{{- end }}
{{- if $probes.readiness }}
readinessProbe:
  httpGet:
    path: {{ $probes.readiness.path | default "/ready" }}
    port: {{ $probes.readiness.port | default "8080" }}
  initialDelaySeconds: {{ $probes.readiness.initialDelaySeconds | default 5 }}
  periodSeconds: {{ $probes.readiness.periodSeconds | default 5 }}
  timeoutSeconds: {{ $probes.readiness.timeoutSeconds | default 3 }}
  failureThreshold: {{ $probes.readiness.failureThreshold | default 3 }}
{{- end }}
{{- if $probes.startup }}
startupProbe:
  httpGet:
    path: {{ $probes.startup.path | default "/startup" }}
    port: {{ $probes.startup.port | default "8080" }}
  initialDelaySeconds: {{ $probes.startup.initialDelaySeconds | default 10 }}
  periodSeconds: {{ $probes.startup.periodSeconds | default 10 }}
  timeoutSeconds: {{ $probes.startup.timeoutSeconds | default 5 }}
  failureThreshold: {{ $probes.startup.failureThreshold | default 30 }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Volume mounts for a specific component
*/}}
{{- define "spec-to-proof.componentVolumeMounts" -}}
{{- $volumeMounts := index .Values .component "volumeMounts" }}
{{- if $volumeMounts }}
volumeMounts:
  {{- range $volumeMounts }}
  - name: {{ .name }}
    mountPath: {{ .mountPath }}
    {{- if .subPath }}
    subPath: {{ .subPath }}
    {{- end }}
    {{- if .readOnly }}
    readOnly: {{ .readOnly }}
    {{- end }}
  {{- end }}
{{- end }}
{{- end }}

{{/*
Volumes for a specific component
*/}}
{{- define "spec-to-proof.componentVolumes" -}}
{{- $volumes := index .Values .component "volumes" }}
{{- if $volumes }}
volumes:
  {{- range $volumes }}
  - name: {{ .name }}
    {{- if .configMap }}
    configMap:
      name: {{ .configMap.name }}
      {{- if .configMap.items }}
      items:
        {{- range .configMap.items }}
        - key: {{ .key }}
          path: {{ .path }}
        {{- end }}
      {{- end }}
    {{- end }}
    {{- if .secret }}
    secret:
      secretName: {{ .secret.secretName }}
      {{- if .secret.items }}
      items:
        {{- range .secret.items }}
        - key: {{ .key }}
          path: {{ .path }}
        {{- end }}
      {{- end }}
    {{- end }}
    {{- if .emptyDir }}
    emptyDir:
      {{- if .emptyDir.medium }}
      medium: {{ .emptyDir.medium }}
      {{- end }}
      {{- if .emptyDir.sizeLimit }}
      sizeLimit: {{ .emptyDir.sizeLimit }}
      {{- end }}
    {{- end }}
    {{- if .persistentVolumeClaim }}
    persistentVolumeClaim:
      claimName: {{ .persistentVolumeClaim.claimName }}
    {{- end }}
  {{- end }}
{{- end }}
{{- end }}

{{/*
Service configuration for a specific component
*/}}
{{- define "spec-to-proof.componentService" -}}
{{- $service := index .Values .component "service" }}
{{- if $service }}
apiVersion: v1
kind: Service
metadata:
  name: {{ include "spec-to-proof.componentServiceName" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) }}
  labels:
    {{- include "spec-to-proof.componentLabels" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) | nindent 4 }}
spec:
  type: {{ $service.type | default "ClusterIP" }}
  ports:
    - port: {{ $service.port | default 8080 }}
      targetPort: {{ $service.targetPort | default $service.port | default 8080 }}
      protocol: TCP
      name: http
  selector:
    {{- include "spec-to-proof.componentSelectorLabels" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) | nindent 4 }}
{{- end }}
{{- end }}

{{/*
HPA configuration for a specific component
*/}}
{{- define "spec-to-proof.componentHPA" -}}
{{- $autoscaling := index .Values .component "autoscaling" }}
{{- if $autoscaling.enabled }}
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ include "spec-to-proof.componentHPAName" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) }}
  labels:
    {{- include "spec-to-proof.componentLabels" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) | nindent 4 }}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ include "spec-to-proof.componentDeploymentName" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) }}
  minReplicas: {{ $autoscaling.minReplicas | default 1 }}
  maxReplicas: {{ $autoscaling.maxReplicas | default 5 }}
  metrics:
    {{- if $autoscaling.targetCPUUtilizationPercentage }}
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: {{ $autoscaling.targetCPUUtilizationPercentage }}
    {{- end }}
    {{- if $autoscaling.targetMemoryUtilizationPercentage }}
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: {{ $autoscaling.targetMemoryUtilizationPercentage }}
    {{- end }}
{{- end }}
{{- end }}

{{/*
PDB configuration for a specific component
*/}}
{{- define "spec-to-proof.componentPDB" -}}
{{- $pdb := index .Values .component "pdb" }}
{{- if $pdb.enabled }}
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: {{ include "spec-to-proof.componentPDBName" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) }}
  labels:
    {{- include "spec-to-proof.componentLabels" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) | nindent 4 }}
spec:
  {{- if $pdb.minAvailable }}
  minAvailable: {{ $pdb.minAvailable }}
  {{- end }}
  {{- if $pdb.maxUnavailable }}
  maxUnavailable: {{ $pdb.maxUnavailable }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "spec-to-proof.componentSelectorLabels" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) | nindent 6 }}
{{- end }}
{{- end }}

{{/*
Network policy configuration for a specific component
*/}}
{{- define "spec-to-proof.componentNetworkPolicy" -}}
{{- $networkPolicy := index .Values .component "networkPolicy" }}
{{- if $networkPolicy.enabled }}
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: {{ include "spec-to-proof.componentNetworkPolicyName" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) }}
  labels:
    {{- include "spec-to-proof.componentLabels" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) | nindent 4 }}
spec:
  podSelector:
    matchLabels:
      {{- include "spec-to-proof.componentSelectorLabels" (dict "component" .component "Values" .Values "Release" .Release "Chart" .Chart) | nindent 6 }}
  policyTypes:
    - Ingress
    - Egress
  ingress:
    {{- range $networkPolicy.ingress }}
    - from:
        {{- range .from }}
        {{- if .podSelector }}
        - podSelector:
            matchLabels:
              {{- range $key, $value := .podSelector.matchLabels }}
              {{ $key }}: {{ $value }}
              {{- end }}
        {{- end }}
        {{- if .namespaceSelector }}
        - namespaceSelector:
            matchLabels:
              {{- range $key, $value := .namespaceSelector.matchLabels }}
              {{ $key }}: {{ $value }}
              {{- end }}
        {{- end }}
        {{- end }}
      ports:
        {{- range .ports }}
        - protocol: {{ .protocol | default "TCP" }}
          port: {{ .port }}
        {{- end }}
    {{- end }}
  egress:
    {{- range $networkPolicy.egress }}
    - to:
        {{- range .to }}
        {{- if .podSelector }}
        - podSelector:
            matchLabels:
              {{- range $key, $value := .podSelector.matchLabels }}
              {{ $key }}: {{ $value }}
              {{- end }}
        {{- end }}
        {{- if .namespaceSelector }}
        - namespaceSelector:
            matchLabels:
              {{- range $key, $value := .namespaceSelector.matchLabels }}
              {{ $key }}: {{ $value }}
              {{- end }}
        {{- end }}
        {{- end }}
      ports:
        {{- range .ports }}
        - protocol: {{ .protocol | default "TCP" }}
          port: {{ .port }}
        {{- end }}
    {{- end }}
{{- end }}
{{- end }} 