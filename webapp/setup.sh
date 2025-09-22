#!/bin/bash

# Startup script to build thread-lens WASM module and setup webapp for deployment
set -e

echo "🚀 Setting up thread-lens webapp..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is not installed. Installing via cargo..."
    cargo install wasm-pack
fi

# Navigate to thread-lens directory and build WASM module
echo "🔨 Building WASM module..."
cd ../thread-lens

# Build the WASM package
wasm-pack build --target web --out-dir pkg

# Return to webapp directory
cd ../webapp

# Clean and create dist directory
echo "📁 Creating dist directory..."
rm -rf dist
mkdir -p dist/thread-lens/pkg

# Copy webapp files to dist
echo "📦 Copying webapp files to dist..."
cp index.html dist/
cp main.js dist/

# Copy the built WASM files to the dist directory
echo "📦 Copying WASM files to dist..."
cp ../thread-lens/pkg/thread_lens.js dist/thread-lens/pkg/
cp ../thread-lens/pkg/thread_lens_bg.wasm dist/thread-lens/pkg/
cp ../thread-lens/pkg/thread_lens.d.ts dist/thread-lens/pkg/ 2>/dev/null || echo "Note: TypeScript definitions not found, skipping..."

# Also setup local development (backward compatibility)
echo "📦 Setting up local development files..."
mkdir -p thread-lens/pkg
cp ../thread-lens/pkg/thread_lens.js thread-lens/pkg/
cp ../thread-lens/pkg/thread_lens_bg.wasm thread-lens/pkg/
cp ../thread-lens/pkg/thread_lens.d.ts thread-lens/pkg/ 2>/dev/null || true

echo "✅ Setup complete!"
echo ""
echo "📁 Distribution files are ready in ./dist/ for CDN deployment:"
echo "  - dist/index.html"
echo "  - dist/main.js"
echo "  - dist/thread-lens/pkg/thread_lens.js"
echo "  - dist/thread-lens/pkg/thread_lens_bg.wasm"
echo ""
echo "🚀 Deploy options:"
echo "  1. Upload the entire 'dist' directory to Netlify, Vercel, or similar CDN"
echo "  2. For local development, serve from current directory:"
echo "     python3 -m http.server 8000"
echo "     # or"
echo "     npx serve ."
echo "  3. For testing the dist build:"
echo "     cd dist && python3 -m http.server 8000"
