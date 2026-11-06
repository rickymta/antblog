targetScope = 'resourceGroup'

param appName string
param location string = resourceGroup().location
param environmentResourceId string
param identityResourceId string
param registryServer string
@description('Immutable registry image reference including @sha256 digest.')
param imageDigest string
@description('Key Vault secret URI for the full PostgreSQL connection string, with TLS VerifyFull.')
param postgresSecretUrl string
param authAuthority string
param authAudience string

resource trips 'Microsoft.App/containerApps@2025-01-01' = {
  name: appName
  location: location
  identity: {
    type: 'UserAssigned'
    userAssignedIdentities: {
      '${identityResourceId}': {}
    }
  }
  properties: {
    managedEnvironmentId: environmentResourceId
    configuration: {
      activeRevisionsMode: 'Single'
      ingress: { external: true, targetPort: 8080, allowInsecure: false }
      registries: [{ server: registryServer, identity: identityResourceId }]
      secrets: [{ name: 'postgres', keyVaultUrl: postgresSecretUrl, identity: identityResourceId }]
    }
    template: {
      scale: { minReplicas: 1, maxReplicas: 3 }
      containers: [{
        name: 'trips'
        image: imageDigest
        resources: { cpu: json('0.5'), memory: '1Gi' }
        env: [
          { name: 'ASPNETCORE_ENVIRONMENT', value: 'Production' }
          { name: 'ASPNETCORE_HTTP_PORTS', value: '8080' }
          { name: 'ConnectionStrings__Trips', secretRef: 'postgres' }
          { name: 'Auth__Authority', value: authAuthority }
          { name: 'Auth__Audience', value: authAudience }
        ]
        probes: [
          { type: 'Startup', httpGet: { path: '/health/live', port: 8080 }, periodSeconds: 2, failureThreshold: 30 }
          { type: 'Liveness', httpGet: { path: '/health/live', port: 8080 }, periodSeconds: 10, failureThreshold: 3 }
          { type: 'Readiness', httpGet: { path: '/health/ready', port: 8080 }, periodSeconds: 10, failureThreshold: 3 }
        ]
      }]
    }
  }
}

output appResourceId string = trips.id
output fqdn string = trips.properties.configuration.ingress.fqdn
