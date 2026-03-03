import sys
import json
import argparse
from typing import Optional
from .patchiest import Patchiest, PatchiestProtocol

def main():
    parser = argparse.ArgumentParser(description="Patchiest: Surgical AST mutations.")
    parser.add_argument(
        "file", 
        nargs="?", 
        help="Path to a JSON file containing the Patchiest action. If omitted, reads from stdin."
    )
    
    args = parser.parse_args()
    
    try:
        if args.file:
            with open(args.file, 'r') as f:
                input_data = f.read()
        else:
            if sys.stdin.isatty():
                parser.print_help()
                return
            input_data = sys.stdin.read()
            
        if not input_data.strip():
            return

        # Parse and validate the action using the protocol
        action_data = json.loads(input_data)
        action = PatchiestProtocol.validate_python(action_data)
        
        # Apply the mutation
        result = Patchiest.apply(action)
        print(result)
        
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON input - {str(e)}")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {str(e)}")
        sys.exit(1)

if __name__ == "__main__":
    main()
