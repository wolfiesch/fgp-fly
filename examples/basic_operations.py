#!/usr/bin/env python3
"""
Fly.io Daemon - Basic Operations Example

Demonstrates common Fly.io operations using the FGP Fly daemon.
Requires:
  - Fly daemon running (`fgp start fly`)
  - FLY_API_TOKEN environment variable set
"""

import json
import socket
import uuid
from pathlib import Path

SOCKET_PATH = Path.home() / ".fgp/services/fly/daemon.sock"


def call_daemon(method: str, params: dict = None) -> dict:
    """Send a request to the Fly daemon and return the response."""
    request = {
        "id": str(uuid.uuid4()),
        "v": 1,
        "method": method,
        "params": params or {}
    }

    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as sock:
        sock.connect(str(SOCKET_PATH))
        sock.sendall((json.dumps(request) + "\n").encode())

        response = b""
        while True:
            chunk = sock.recv(4096)
            if not chunk:
                break
            response += chunk
            if b"\n" in response:
                break

        return json.loads(response.decode().strip())


def list_apps():
    """List all Fly.io applications."""
    print("\n✈️ Fly.io Applications")
    print("-" * 40)

    result = call_daemon("fly.apps", {})

    if result.get("ok"):
        apps = result["result"].get("apps", [])
        if not apps:
            print("  No applications found")
        for app in apps:
            status = app.get("status", "unknown")
            status_icon = "🟢" if status == "deployed" else "🔴" if status == "suspended" else "🟡"
            print(f"  {status_icon} {app.get('name')}")
            print(f"     Organization: {app.get('organization', {}).get('slug', 'personal')}")
            print(f"     Status: {status}")
            print()
    else:
        print(f"  ❌ Error: {result.get('error')}")


def get_app_status(app_name: str):
    """Get detailed status for a specific app."""
    print(f"\n📊 Status for: {app_name}")
    print("-" * 40)

    result = call_daemon("fly.status", {"app": app_name})

    if result.get("ok"):
        status = result["result"]
        print(f"  Name: {status.get('name')}")
        print(f"  Status: {status.get('status')}")
        print(f"  Hostname: {status.get('hostname')}")

        machines = status.get("machines", [])
        print(f"\n  Machines ({len(machines)}):")
        for machine in machines:
            state = machine.get("state", "unknown")
            state_icon = "🟢" if state == "started" else "🔴"
            print(f"    {state_icon} {machine.get('id')} - {machine.get('region')} ({state})")
    else:
        print(f"  ❌ Error: {result.get('error')}")


def list_machines(app_name: str):
    """List all machines for an app."""
    print(f"\n🖥️ Machines for: {app_name}")
    print("-" * 40)

    result = call_daemon("fly.machines", {"app": app_name})

    if result.get("ok"):
        machines = result["result"].get("machines", [])
        if not machines:
            print("  No machines found")
        for machine in machines:
            state = machine.get("state", "unknown")
            state_icon = "🟢" if state == "started" else "🔴" if state == "stopped" else "🟡"
            print(f"  {state_icon} {machine.get('id')}")
            print(f"     Region: {machine.get('region')}")
            print(f"     State: {state}")
            print(f"     Created: {machine.get('created_at', 'unknown')}")
            print()
    else:
        print(f"  ❌ Error: {result.get('error')}")


def scale_machine(app_name: str, machine_id: str, action: str):
    """Scale a machine (start/stop).

    Args:
        app_name: The application name
        machine_id: The machine ID to scale
        action: "start" or "stop"
    """
    print(f"\n⚡ {action.capitalize()}ing machine: {machine_id}")

    result = call_daemon("fly.scale", {
        "app": app_name,
        "machine_id": machine_id,
        "action": action
    })

    if result.get("ok"):
        print(f"  ✅ Machine {action} initiated")
    else:
        print(f"  ❌ Error: {result.get('error')}")


if __name__ == "__main__":
    print("Fly.io Daemon Examples")
    print("=" * 40)

    # Check daemon health first
    health = call_daemon("health")
    if not health.get("ok"):
        print("❌ Fly daemon not running. Start with: fgp start fly")
        print("   Also ensure FLY_API_TOKEN is set")
        exit(1)

    print("✅ Fly daemon is healthy")

    # Run examples
    list_apps()

    # Uncomment to check a specific app:
    # get_app_status("your-app-name")
    # list_machines("your-app-name")

    # Uncomment to scale a machine:
    # scale_machine("your-app-name", "machine-id", "stop")
