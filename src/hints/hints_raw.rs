pub const STARKNET_OS_INPUT: &str =
    "from starkware.starknet.core.os.os_input import StarknetOsInput\n\nos_input = \
     StarknetOsInput.load(data=program_input)\n\nids.initial_carried_outputs.messages_to_l1 = \
     segments.add_temp_segment()\nids.initial_carried_outputs.messages_to_l2 = segments.add_temp_segment()";

pub const LOAD_CLASS_FACTS: &str = "ids.compiled_class_facts = segments.add()\nids.n_compiled_class_facts = \
                                    len(os_input.compiled_classes)\nvm_enter_scope({\n    'compiled_class_facts': \
                                    iter(os_input.compiled_classes.items()),\n})";

pub const LOAD_DEPRECATED_CLASS_FACTS: &str =
    "# Creates a set of deprecated class hashes to distinguish calls to deprecated entry \
     points.\n__deprecated_class_hashes=set(os_input.deprecated_compiled_classes.keys())\nids.compiled_class_facts = \
     segments.add()\nids.n_compiled_class_facts = len(os_input.deprecated_compiled_classes)\nvm_enter_scope({\n    \
     'compiled_class_facts': iter(os_input.deprecated_compiled_classes.items()),\n})";

pub const LOAD_DEPRECATED_CLASS_INNER: &str =
    "from starkware.starknet.core.os.contract_class.deprecated_class_hash import (\n    \
     get_deprecated_contract_class_struct,\n)\n\ncompiled_class_hash, compiled_class = \
     next(compiled_class_facts)\n\ncairo_contract = get_deprecated_contract_class_struct(\n    \
     identifiers=ids._context.identifiers, contract_class=compiled_class)\nids.compiled_class = \
     segments.gen_arg(cairo_contract)";

pub const CHECK_DEPRECATED_CLASS_HASH: &str =
    "from starkware.python.utils import from_bytes\n\ncomputed_hash = ids.compiled_class_fact.hash\nexpected_hash = \
     compiled_class_hash\nassert computed_hash == expected_hash, (\n    \"Computed compiled_class_hash is \
     inconsistent with the hash in the os_input. \"\n    f\"Computed hash = {computed_hash}, Expected hash = \
     {expected_hash}.\")\n\nvm_load_program(compiled_class.program, ids.compiled_class.bytecode_ptr)";

/// This is the equivalent of nondet %{ os_input.general_config.sequencer_address %}
pub const SEQUENCER_ADDRESS: &str = "memory[ap] = to_felt_or_relocatable(os_input.general_config.sequencer_address)";

pub const DEPRECATED_BLOCK_NUMBER: &str =
    "memory[ap] = to_felt_or_relocatable(deprecated_syscall_handler.block_info.block_number)";

pub const DEPRECATED_BLOCK_TIMESTAMP: &str =
    "memory[ap] = to_felt_or_relocatable(deprecated_syscall_handler.block_info.block_timestamp)";

pub const CHAIN_ID: &str = "memory[ap] = to_felt_or_relocatable(os_input.general_config.chain_id.value)";

pub const FEE_TOKEN_ADDRESS: &str = "memory[ap] = to_felt_or_relocatable(os_input.general_config.fee_token_address)";

pub const INITIALIZE_STATE_CHANGES: &str = "from starkware.python.utils import from_bytes\n\ninitial_dict = {\n    \
                                            address: segments.gen_arg(\n        (from_bytes(contract.contract_hash), \
                                            segments.add(), contract.nonce))\n    for address, contract in \
                                            os_input.contracts.items()\n}";

pub const INITIALIZE_CLASS_HASHES: &str = "initial_dict = os_input.class_hash_to_compiled_class_hash";

pub const GET_BLOCK_MAPPING: &str =
    "ids.state_entry = __dict_manager.get_dict(ids.contract_state_changes)[\n    ids.BLOCK_HASH_CONTRACT_ADDRESS\n]";

pub const SEGMENTS_ADD: &str = "memory[ap] = to_felt_or_relocatable(segments.add())";

pub const SEGMENTS_ADD_TEMP: &str = "memory[ap] = to_felt_or_relocatable(segments.add_temp_segment())";

pub const TRANSACTIONS_LEN: &str = "memory[ap] = to_felt_or_relocatable(len(os_input.transactions))";

pub const ENTER_SYSCALL_SCOPES: &str =
    "vm_enter_scope({\n    '__deprecated_class_hashes': __deprecated_class_hashes,\n    'transactions': \
     iter(os_input.transactions),\n    'execution_helper': execution_helper,\n    'deprecated_syscall_handler': \
     deprecated_syscall_handler,\n    'syscall_handler': syscall_handler,\n     '__dict_manager': __dict_manager,\n})";

pub const LOAD_NEXT_TX: &str = "tx = next(transactions)\ntx_type_bytes = \
                                tx.tx_type.name.encode(\"ascii\")\nids.tx_type = int.from_bytes(tx_type_bytes, \
                                \"big\")";

pub const LOAD_CONTRACT_ADDRESS: &str = "from starkware.starknet.business_logic.transaction.objects import \
                                         InternalL1Handler\nids.contract_address = (\ntx.contract_address if \
                                         isinstance(tx, InternalL1Handler) else tx.sender_address\n)";

pub const PREPARE_CONSTRUCTOR_EXECUTION: &str = "ids.contract_address_salt = tx.contract_address_salt\nids.class_hash \
                                                 = tx.class_hash\nids.constructor_calldata_size = \
                                                 len(tx.constructor_calldata)\nids.constructor_calldata = \
                                                 segments.gen_arg(arg=tx.constructor_calldata)";

pub const TRANSACTION_VERSION: &str = "memory[ap] = to_felt_or_relocatable(tx.version)";

pub const ASSERT_TRANSACTION_HASH: &str =
    "assert ids.transaction_hash == tx.hash_value, (\n    \"Computed transaction_hash is inconsistent with the hash \
     in the transaction. \"\n    f\"Computed hash = {ids.transaction_hash}, Expected hash = {tx.hash_value}.\")";

pub const FORMAT_OS_OUTPUT: &str =
     "from starkware.python.math_utils import div_ceil\nonchain_data_start = ids.da_start\nonchain_data_size = ids.output_ptr - onchain_data_start\n\nmax_page_size = 3800\nn_pages = div_ceil(onchain_data_size, max_page_size)\nfor i in range(n_pages):\n    start_offset = i * max_page_size\n    output_builtin.add_page(\n        page_id=1 + i,\n        page_start=onchain_data_start + start_offset,\n        page_size=min(onchain_data_size - start_offset, max_page_size),\n    )\n# Set the tree structure to a root with two children:\n# * A leaf which represents the main part\n# * An inner node for the onchain data part (which contains n_pages children).\n#\n# This is encoded using the following sequence:\noutput_builtin.add_attribute('gps_fact_topology', [\n    # Push 1 + n_pages pages (all of the pages).\n    1 + n_pages,\n    # Create a parent node for the last n_pages.\n    n_pages,\n    # Don't push additional pages.\n    0,\n    # Take the first page (the main part) and the node that was created (onchain data)\n    # and use them to construct the root of the fact tree.\n    2,\n])";

pub const START_DEPLOY_TX: &str =
    "execution_helper.start_tx(\n    tx_info_ptr=ids.constructor_execution_context.deprecated_tx_info.address_\n)";

pub const GET_STATE_ENTRY: &str = "# Fetch a state_entry in this hint and validate it in the update at the end\n# of \
                                   this function.\nids.state_entry = \
                                   __dict_manager.get_dict(ids.contract_state_changes)[ids.contract_address]";

pub const CHECK_IS_DEPRECATED: &str =
    "is_deprecated = 1 if ids.execution_context.class_hash in __deprecated_class_hashes else 0";

pub const IS_DEPRECATED: &str = "memory[ap] = to_felt_or_relocatable(is_deprecated)";

pub const OS_CONTEXT_SEGMENTS: &str = "ids.os_context = segments.add()\nids.syscall_ptr = segments.add()";

pub const SELECTED_BUILTINS: &str = "vm_enter_scope({'n_selected_builtins': ids.n_selected_builtins})";

pub const SELECT_BUILTIN: &str =
    "# A builtin should be selected iff its encoding appears in the selected encodings list\n# and the list wasn't \
     exhausted.\n# Note that testing inclusion by a single comparison is possible since the lists are \
     sorted.\nids.select_builtin = int(\n  n_selected_builtins > 0 and memory[ids.selected_encodings] == \
     memory[ids.all_encodings])\nif ids.select_builtin:\n  n_selected_builtins = n_selected_builtins - 1";

pub const ENTER_CALL: &str =
    "execution_helper.enter_call(\n    execution_info_ptr=ids.execution_context.execution_info.address_)";

pub const ENTER_SCOPE_SYSCALL_HANDLER: &str = "vm_enter_scope({'syscall_handler': deprecated_syscall_handler})";
pub const BREAKPOIN: &str = "breakpoint()";
