# Dimension Bridge - Deployment Guide

## 🚀 Image Management & Deployment Strategy

### Image Build and Distribution Process

Dimension Bridge uses the following image management strategy:

#### 1. Centralized Image Building

- **Administrator/DevOps team** builds images centrally
- **Fixed image tags** used: `dimension-bridge:v1.0.0`
- Same image used across all environments

#### 2. Image Distribution Methods

#### A. Container Registry (Recommended)

```bash
# 1. Administrator builds image
docker build -t your-registry.com/dimension-bridge:v1.0.0 .

# 2. Push to registry
docker push your-registry.com/dimension-bridge:v1.0.0

# 3. Pull in each environment
docker pull your-registry.com/dimension-bridge:v1.0.0
```

#### B. Local Image Distribution

```bash
# 1. Administrator builds image
docker build -t dimension-bridge:v1.0.0 .

# 2. Save image
docker save dimension-bridge:v1.0.0 > dimension-bridge-v1.0.0.tar

# 3. Transfer to each server and load
docker load < dimension-bridge-v1.0.0.tar
```

## 📋 Docker Compose Usage

### Currently Configured Image Tag

All examples use the following image:

```yaml
image: dimension-bridge:v1.0.0
```

### Environment-specific Configuration

#### Development Environment

```bash
# Create .env file
echo "DIMENSION_BRIDGE_IMAGE=dimension-bridge:v1.0.0-dev" > .env

# Use in docker-compose
docker-compose up -d
```

#### Staging Environment

```bash
echo "DIMENSION_BRIDGE_IMAGE=dimension-bridge:v1.0.0-staging" > .env
docker-compose up -d
```

#### Production Environment

```bash
echo "DIMENSION_BRIDGE_IMAGE=dimension-bridge:v1.0.0" > .env
docker-compose up -d
```

## 🔧 Prerequisites

### 1. Image Verification

Always verify the image exists before use:

```bash
# Check local images
docker images | grep dimension-bridge

# Expected output:
# dimension-bridge    v1.0.0    abc123def456    2 hours ago    45.2MB
```

### 2. What to do if Image is Missing

#### A. Request from Administrator

```bash
# Request from administrator with following info
echo "Required image: dimension-bridge:v1.0.0"
echo "Target server: $(hostname)"
echo "Docker version: $(docker --version)"
```

#### B. Emergency Local Build (Development Only)

```bash
# Use only in emergencies (NOT for production)
docker build -t dimension-bridge:v1.0.0 .
```

## 🚨 Important Guidelines

### What NOT to Do

❌ **Using local builds in production**

```yaml
# DON'T do this!
build:
  context: .
  dockerfile: Dockerfile
```

❌ **Using latest tag**

```yaml
# DON'T do this!
image: dimension-bridge:latest
```

❌ **Arbitrary image tag changes**

```yaml
# DON'T do this!
image: dimension-bridge:my-custom-version
```

### What you MUST Do

✅ **Use fixed version tags**

```yaml
image: dimension-bridge:v1.0.0
```

✅ **Verify image exists before deployment**

```bash
docker images dimension-bridge:v1.0.0
```

✅ **Manage image tags with environment variables**

```yaml
image: ${DIMENSION_BRIDGE_IMAGE:-dimension-bridge:v1.0.0}
```

## 🔄 Version Update Process

### 1. New Version Release

**Administrator Tasks:**

```bash
# Build new version
docker build -t dimension-bridge:v1.1.0 .
docker tag dimension-bridge:v1.1.0 dimension-bridge:latest

# Deploy (if using registry)
docker push your-registry.com/dimension-bridge:v1.1.0
```

**User Tasks:**

```bash
# 1. Pull new image
docker pull dimension-bridge:v1.1.0

# 2. Update environment variable
echo "DIMENSION_BRIDGE_IMAGE=dimension-bridge:v1.1.0" > .env

# 3. Restart services
docker-compose pull cert-manager
docker-compose up -d cert-manager
```

### 2. Rollback Procedure

```bash
# Rollback to previous version
echo "DIMENSION_BRIDGE_IMAGE=dimension-bridge:v1.0.0" > .env
docker-compose up -d cert-manager
```

## 📊 Image Status Monitoring

### Regular Health Check Script

```bash
#!/bin/bash
# image-check.sh

REQUIRED_IMAGE="dimension-bridge:v1.0.0"

echo "=== Dimension Bridge Image Status Check ==="
echo "Required image: $REQUIRED_IMAGE"
echo

# Check image existence
if docker images --format "table {{.Repository}}:{{.Tag}}" | grep -q "$REQUIRED_IMAGE"; then
    echo "✅ Image exists"
    docker images | grep dimension-bridge
else
    echo "❌ Image not found!"
    echo "Please request this image from administrator: $REQUIRED_IMAGE"
    exit 1
fi

echo
echo "=== Running Container Check ==="
docker ps --format "table {{.Names}}\t{{.Image}}\t{{.Status}}" | grep dimension-bridge
```

## 🆘 Troubleshooting

### Common Issues

#### Issue 1: Image not found

```bash
Error: Unable to find image 'dimension-bridge:v1.0.0' locally
```

**Solution:**

```bash
# Check image list
docker images | grep dimension-bridge

# Request image from administrator or
# Pull from registry
docker pull your-registry.com/dimension-bridge:v1.0.0
docker tag your-registry.com/dimension-bridge:v1.0.0 dimension-bridge:v1.0.0
```

#### Issue 2: Container won't start

```bash
# Check logs
docker-compose logs cert-manager

# Check image version
docker inspect dimension-bridge:v1.0.0
```

#### Issue 3: Wrong image version in use

```bash
# Check currently used image
docker-compose ps

# Replace with correct image
docker-compose pull
docker-compose up -d
```

## 📞 Support

When reporting issues, provide the following information:

- Server info: `hostname`, `docker --version`
- Image status: `docker images | grep dimension-bridge`
- Error logs: `docker-compose logs cert-manager`
- Environment config: `.env` file contents (excluding sensitive data)
