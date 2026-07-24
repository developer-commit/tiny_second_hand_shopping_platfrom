import subprocess
import sys

def run_script(script_name):
    print(f"\n======================================")
    print(f"Running {script_name}...")
    print(f"======================================")
    result = subprocess.run([sys.executable, script_name])
    if result.returncode != 0:
        print(f"\n❌ {script_name} failed!")
        return False
    print(f"\n✅ {script_name} completed successfully.")
    return True

if __name__ == "__main__":
    scripts = [
        "scenario_1_auth_users.py",
        "scenario_2_products_chat.py",
        "scenario_3_wallet_escrow.py",
    ]
    
    for script in scripts:
        if not run_script(script):
            print("\n🚨 Test suite aborted due to failure.")
            sys.exit(1)
            
    print("\n🎉 All test scenarios passed successfully!")
    sys.exit(0)
