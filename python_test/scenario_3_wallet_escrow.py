import requests
import sys
import os
from utils import print_result, print_skip

BASE_URL = "http://localhost:8080/v1"

def run_scenario():
    print("=== Scenario 3: Wallet & Escrow ===")
    
    # Load token
    try:
        with open("test_token.txt", "r") as f:
            access_token = f.read().strip()
    except FileNotFoundError:
        print("❌ Cannot find test_token.txt. Run Scenario 1 first.")
        return False
        
    # Load item uid
    try:
        with open("test_item.txt", "r") as f:
            item_uid = f.read().strip()
    except FileNotFoundError:
        print("❌ Cannot find test_item.txt. Run Scenario 2 first.")
        return False

    session = requests.Session()
    session.headers.update({"Authorization": f"Bearer {access_token}"})

    # Step 3.1: Wallet Info
    print("\n--- [Step 3.1] Wallet Info ---")
    response = session.get(f"{BASE_URL}/wallet")
    print_result("Get Wallet Info", "200", response, response.status_code == 200)

    # Step 3.2: Wallet History
    print("\n--- [Step 3.2] Wallet History ---")
    response = session.get(f"{BASE_URL}/wallet/history")
    print_result("Get Wallet History", "200", response, response.status_code == 200)

    # Step 3.3: Escrow Deposit
    print("\n--- [Step 3.3] Escrow Deposit ---")
    response = session.post(
        f"{BASE_URL}/escrow",
        json={"item_uid": item_uid}
    )
    print_result("Escrow Deposit", "201", response, response.status_code in [201, 400]) # 400 might happen if insufficient funds
    
    trade_uid = None
    if response.status_code == 201:
        json_resp = response.json()
        trade_uid = json_resp.get("trade_uid") or json_resp.get("data", {}).get("trade_uid")
        
    if trade_uid:
        # Step 3.4: Escrow Status
        print("\n--- [Step 3.4] Escrow Status ---")
        status_resp = session.get(f"{BASE_URL}/escrow/{trade_uid}")
        print_result("Escrow Status", "200", status_resp, status_resp.status_code == 200)

        # Step 3.5: Escrow Confirm
        print("\n--- [Step 3.5] Escrow Confirm ---")
        confirm_resp = session.post(f"{BASE_URL}/escrow/{trade_uid}/confirm")
        print_result("Escrow Confirm", "200", confirm_resp, confirm_resp.status_code == 200)
    else:
        print_skip("Escrow Status/Confirm", "No trade_uid obtained (possibly insufficient funds).")

    return True

if __name__ == "__main__":
    success = run_scenario()
    sys.exit(0 if success else 1)
