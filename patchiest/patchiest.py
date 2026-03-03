import ast
from pathlib import Path
from typing import List, Optional, Dict, Any, Union, Literal
from pydantic import BaseModel, Field, TypeAdapter

# --- 1. THE NATIVE PROTOCOL SCHEMA ---

class MutationBase(BaseModel):
    pass

class TranslateDialectMutations(MutationBase):
    enforceExplicitType: str
    targetParamIndex: int = 0

class RestructureTopologyMutations(BaseModel):
    extractToParameter: str

class MutateCallMutations(BaseModel):
    rename: Optional[str] = None
    injectArgs: Optional[Dict[str, Any]] = None

class ManageImportMutations(BaseModel):
    replaceWith: Optional[str] = None
    moduleSpecifier: Optional[str] = None

class ActionBase(BaseModel):
    file_path: str
    target: Dict[str, str]

class TranslateDialectAction(ActionBase):
    action: Literal['TRANSLATE_DIALECT']
    mutations: TranslateDialectMutations

class RestructureTopologyAction(ActionBase):
    action: Literal['RESTRUCTURE_TOPOLOGY']
    mutations: RestructureTopologyMutations

class MutateCallAction(ActionBase):
    action: Literal['MUTATE_CALL']
    mutations: MutateCallMutations

class ManageImportAction(ActionBase):
    action: Literal['MANAGE_IMPORT']
    mutations: ManageImportMutations

# The Strict Union for the Alchemist Protocol
DialecticAction = Union[
    TranslateDialectAction, RestructureTopologyAction, 
    MutateCallAction, ManageImportAction
]

# Using TypeAdapter to correctly generate JSON schemas for Unions
PatchiestProtocol = TypeAdapter(DialecticAction)

# --- 2. THE NATIVE MUTATION ENGINE ---

class Patchiest:
    """Native Python implementation of the Patchiest AST Protocol."""

    @staticmethod
    def apply(action: DialecticAction) -> str:
        """Executes a surgical mutation on a Python source file."""
        path = Path(action.file_path)
        if not path.exists():
            return f"Error: File '{action.file_path}' not found."

        try:
            source = path.read_text()
            tree = ast.parse(source)
            modified = False

            if action.action == 'MUTATE_CALL':
                target_name = action.target.get("name")
                for node in ast.walk(tree):
                    if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
                        if node.func.id == target_name:
                            if action.mutations.rename:
                                node.func.id = action.mutations.rename
                                modified = True

            if modified:
                path.write_text(ast.unparse(tree))
                return f"SUCCESS: Applied {action.action} to {action.file_path}."
            
            return f"NO_MUTATIONS: {action.action} target not found in {action.file_path}."

        except Exception as e:
            # Zero-Tolerance Transactional Rollback: Path remains unchanged on failure
            return f"ROLLBACK: Critical failure in {action.action}: {str(e)}."