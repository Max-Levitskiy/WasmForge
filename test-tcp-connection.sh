#!/bin/bash

# Test script for remote TCP connection to WasmForge MCP server
# Usage: ./test-tcp-connection.sh [host] [port]

HOST=${1:-127.0.0.1}
PORT=${2:-8080}

echo "Testing WasmForge MCP Server at $HOST:$PORT"
echo "========================================"

echo -e "\n1. Testing initialize:"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | nc $HOST $PORT

echo -e "\n2. Testing tools/list:"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | nc $HOST $PORT

echo -e "\n3. Testing add tool:"
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"add","arguments":{"a":25,"b":17}}}' | nc $HOST $PORT

echo -e "\n4. Testing fetch tool:"
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"fetch","arguments":{"url":"https://api.github.com/zen"}}}' | nc $HOST $PORT

echo -e "\n5. Testing invalid URL:"
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"fetch","arguments":{"url":"not-a-url"}}}' | nc $HOST $PORT

echo -e "\nDone!"