#!/bin/bash

set -e

# Function to detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)
    
    case $OS in
        linux)
            case $ARCH in
                x86_64) PLATFORM="linux-x86_64" ;;
                aarch64) PLATFORM="linux-aarch64" ;;
                *) echo "Unsupported Linux architecture: $ARCH"; exit 1 ;;
            esac
            ;;
        darwin)
            case $ARCH in
                x86_64) PLATFORM="mac-x86_64" ;;
                arm64) PLATFORM="mac-arm64" ;;
                *) echo "Unsupported macOS architecture: $ARCH"; exit 1 ;;
            esac
            ;;
        *) echo "Unsupported OS: $OS"; exit 1 ;;
    esac
    
    echo $PLATFORM
}

# Detect platform
PLATFORM=$(detect_platform)

# Set HWI version
HWI_VERSION="2.3.1"

# Set download URL
DOWNLOAD_URL="https://github.com/bitcoin-core/HWI/releases/download/${HWI_VERSION}/hwi-${HWI_VERSION}-${PLATFORM}.tar.gz"

# Set output directory
OUTPUT_DIR="hwi-binary"

# Create output directory if it doesn't exist
mkdir -p $OUTPUT_DIR
chmod 755 $OUTPUT_DIR

# Download and extract HWI
echo "Downloading HWI for $PLATFORM..."
curl -L $DOWNLOAD_URL -o hwi.tar.gz
echo "Download completed. Extracting..."
chmod 644 hwi.tar.gz
tar xzvf hwi.tar.gz -C $OUTPUT_DIR
rm hwi.tar.gz

# Make the binary executable
chmod +x $OUTPUT_DIR/hwi

echo "HWI binary downloaded and extracted to $OUTPUT_DIR/hwi"
