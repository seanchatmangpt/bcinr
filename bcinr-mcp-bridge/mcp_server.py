import sys
import subprocess
import json
import threading
import time

lsp_proc = None
req_id = 1
response_futures = {}

def log(msg):
    # MCP standard says stderr is for logging
    sys.stderr.write(msg + "\n")
    sys.stderr.flush()

def start_lsp():
    global lsp_proc
    binary_path = "/Users/sac/bcinr/target/debug/bcinr-pddl-lsp"
    
    lsp_proc = subprocess.Popen(
        [binary_path],
        cwd="/Users/sac/bcinr",
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=sys.stderr,
        text=False
    )
    
    threading.Thread(target=read_lsp, daemon=True).start()
    
    # Initialize LSP
    resp = send_lsp_request("initialize", {
        "processId": None,
        "rootUri": "file:///Users/sac/bcinr",
        "capabilities": {}
    })
    send_lsp_notification("initialized", {})
    return resp

def read_lsp():
    while True:
        content_length = 0
        while True:
            line = lsp_proc.stdout.readline()
            if not line: return
            line = line.decode('utf-8')
            if line == '\r\n':
                break
            if line.startswith('Content-Length:'):
                content_length = int(line.split(':')[1].strip())
        
        body = lsp_proc.stdout.read(content_length)
        if not body: return
        data = json.loads(body.decode('utf-8'))
        
        if "id" in data and data["id"] in response_futures:
            response_futures[data["id"]] = data

def send_lsp_request(method, params):
    global req_id
    id_ = req_id
    req_id += 1
    req = {
        "jsonrpc": "2.0",
        "id": id_,
        "method": method,
        "params": params
    }
    body = json.dumps(req).encode('utf-8')
    header = f"Content-Length: {len(body)}\r\n\r\n".encode('utf-8')
    lsp_proc.stdin.write(header + body)
    lsp_proc.stdin.flush()
    
    for _ in range(50):
        if id_ in response_futures:
            return response_futures.pop(id_)
        time.sleep(0.1)
    return {"error": "timeout"}

def send_lsp_notification(method, params):
    req = {
        "jsonrpc": "2.0",
        "method": method,
        "params": params
    }
    body = json.dumps(req).encode('utf-8')
    header = f"Content-Length: {len(body)}\r\n\r\n".encode('utf-8')
    lsp_proc.stdin.write(header + body)
    lsp_proc.stdin.flush()

# --- MCP Protocol Implementation ---

def read_mcp_message():
    line = sys.stdin.readline()
    if not line: return None
    try:
        return json.loads(line)
    except:
        return None

def send_mcp_response(id_, result=None, error=None):
    msg = {"jsonrpc": "2.0", "id": id_}
    if error:
        msg["error"] = error
    else:
        msg["result"] = result
    sys.stdout.write(json.dumps(msg) + "\n")
    sys.stdout.flush()

def handle_mcp():
    start_lsp()
    
    while True:
        req = read_mcp_message()
        if not req:
            break
        
        method = req.get("method")
        id_ = req.get("id")
        params = req.get("params", {})
        
        if method == "initialize":
            send_mcp_response(id_, {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "serverInfo": {"name": "bcinr-mcp-bridge", "version": "1.0.0"}
            })
        elif method == "notifications/initialized":
            pass
        elif method == "tools/list":
            send_mcp_response(id_, {
                "tools": [
                    {
                        "name": "bcinr_request_build_slot",
                        "description": "Request a build slot for a heavy command",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "command": {"type": "string"}
                            },
                            "required": ["command"]
                        }
                    },
                    {
                        "name": "bcinr_release_build_slot",
                        "description": "Release the build slot",
                        "inputSchema": {
                            "type": "object",
                            "properties": {},
                            "required": []
                        }
                    },
                    {
                        "name": "bcinr_execute_tape",
                        "description": "Execute the plan tape and admit the result",
                        "inputSchema": {
                            "type": "object",
                            "properties": {},
                            "required": []
                        }
                    },
                    {
                        "name": "bcinr_read_virtual_doc",
                        "description": "Read a virtual document (e.g. bcinr-pddl://status)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "uri": {"type": "string"}
                            },
                            "required": ["uri"]
                        }
                    }
                ]
            })
        elif method == "tools/call":
            tool_name = params.get("name")
            tool_args = params.get("arguments", {})
            
            resp_content = ""
            if tool_name == "bcinr_request_build_slot":
                resp = send_lsp_request("workspace/executeCommand", {
                    "command": "bcinrPddl.requestBuildSlot",
                    "arguments": [tool_args.get("command", "build")]
                })
                resp_content = json.dumps(resp)
            elif tool_name == "bcinr_release_build_slot":
                resp = send_lsp_request("workspace/executeCommand", {
                    "command": "bcinrPddl.releaseBuildSlot",
                    "arguments": []
                })
                resp_content = json.dumps(resp)
            elif tool_name == "bcinr_execute_tape":
                resp = send_lsp_request("workspace/executeCommand", {
                    "command": "bcinrPddl.executeTape",
                    "arguments": []
                })
                resp_content = json.dumps(resp)
            elif tool_name == "bcinr_read_virtual_doc":
                resp = send_lsp_request("workspace/executeCommand", {
                    "command": "bcinrPddl.openVirtualDocument",
                    "arguments": [tool_args.get("uri", "bcinr-pddl://status")]
                })
                resp_content = json.dumps(resp)
            else:
                send_mcp_response(id_, error={"code": -32601, "message": "Method not found"})
                continue
                
            send_mcp_response(id_, {
                "content": [{"type": "text", "text": resp_content}]
            })
        else:
            if id_ is not None:
                send_mcp_response(id_, error={"code": -32601, "message": "Method not found"})

if __name__ == "__main__":
    handle_mcp()
