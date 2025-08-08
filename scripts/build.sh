#!/bin/bash
# Build script for Dimension Bridge Certificate Manager

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
IMAGE_NAME=${IMAGE_NAME:-"appleparan/dimension-bridge"}
TAG=${TAG:-"latest"}
BUILD_ARGS=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --tag|-t)
            TAG="$2"
            shift 2
            ;;
        --image|-i)
            IMAGE_NAME="$2"
            shift 2
            ;;
        --platform)
            BUILD_ARGS="$BUILD_ARGS --platform $2"
            shift 2
            ;;
        --push)
            BUILD_ARGS="$BUILD_ARGS --push"
            shift
            ;;
        --no-cache)
            BUILD_ARGS="$BUILD_ARGS --no-cache"
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --tag, -t TAG          Docker image tag (default: latest)"
            echo "  --image, -i IMAGE      Docker image name (default: appleparan/dimension-bridge)"
            echo "  --platform PLATFORM   Target platform (e.g., linux/amd64,linux/arm64)"
            echo "  --push                 Push image to registry"
            echo "  --no-cache            Build without using cache"
            echo "  --help, -h            Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                                    # Build with default settings"
            echo "  $0 --tag v1.0.0 --push              # Build and push version v1.0.0"
            echo "  $0 --platform linux/amd64,linux/arm64 --push # Multi-platform build"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}Building Dimension Bridge Certificate Manager${NC}"
echo -e "${YELLOW}Image: ${IMAGE_NAME}:${TAG}${NC}"

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker is not installed or not in PATH${NC}"
    exit 1
fi

# Check if we're in the right directory
if [[ ! -f "Dockerfile" ]]; then
    echo -e "${RED}Error: Dockerfile not found. Please run this script from the project root.${NC}"
    exit 1
fi

# Run linting and tests before building
echo -e "${YELLOW}Running pre-build checks...${NC}"

# Check if cargo is available for local testing
if command -v cargo &> /dev/null; then
    echo "Running cargo fmt check..."
    cargo fmt --check || {
        echo -e "${RED}Code formatting check failed. Run 'cargo fmt' to fix.${NC}"
        exit 1
    }

    echo "Running cargo clippy..."
    cargo clippy -- -D warnings || {
        echo -e "${RED}Clippy check failed. Please fix the warnings.${NC}"
        exit 1
    }

    echo "Running cargo check..."
    cargo check || {
        echo -e "${RED}Cargo check failed. Please fix compilation errors.${NC}"
        exit 1
    }
else
    echo -e "${YELLOW}Cargo not found, skipping local tests. Docker build will catch any issues.${NC}"
fi

# Build the Docker image
echo -e "${YELLOW}Building Docker image...${NC}"
docker buildx build \
    --tag "${IMAGE_NAME}:${TAG}" \
    --tag "${IMAGE_NAME}:latest" \
    $BUILD_ARGS \
    .

if [[ $? -eq 0 ]]; then
    echo -e "${GREEN}✓ Docker image built successfully: ${IMAGE_NAME}:${TAG}${NC}"

    # Test the built image
    echo -e "${YELLOW}Testing the built image...${NC}"
    docker run --rm "${IMAGE_NAME}:${TAG}" version

    if [[ $? -eq 0 ]]; then
        echo -e "${GREEN}✓ Image test passed${NC}"
        echo ""
        echo -e "${GREEN}Build completed successfully!${NC}"
        echo "Image: ${IMAGE_NAME}:${TAG}"
        echo ""
        echo "To run the certificate manager:"
        echo "  docker run --rm \\"
        echo "    -e CERT_DOMAINS=example.com \\"
        echo "    -e STEP_CA_URL=https://ca.example.com:9000 \\"
        echo "    -e RELOAD_COMMAND='echo reload' \\"
        echo "    -v /tmp/certs:/certs \\"
        echo "    ${IMAGE_NAME}:${TAG}"
    else
        echo -e "${RED}✗ Image test failed${NC}"
        exit 1
    fi
else
    echo -e "${RED}✗ Docker build failed${NC}"
    exit 1
fi
