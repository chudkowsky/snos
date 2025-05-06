
## Algorithm for Filtering the State Update

### High-Level Goal

The algorithm's purpose is to:
- **Filter** the contract state changes so that only relevant storage slots (keys) are included in the output.
- **Format** these filtered changes for on-chain data availability (DA).
- **Calculate** a Merkle root over the filtered output for integrity and succinctness.

---

### Step-by-Step Algorithm
#### High level overview of outputing :
```
main{
    input;
    reexecution of txs
    output
}
```

serialization flow of state diff structure:  

output proccess
```
serialize_os_output -> output_contract_state -> 
output_contract_state_inner -> serialize_da_changes -> serialize_da_changes_inner
``` 


#### 1. Preparation

- The function `output_contract_state` is called with:
  - All contract state changes (`contract_state_changes_start`)
  - A list of storage keys of interest (`ptr_to_storage_keys`)
  - Other metadata (number of changes, flags, etc.)

#### 2. Filtering Relevant Slots
- An empty array `filtered_slots` is allocated.
- The function `output_contract_state_inner` is called recursively for each contract:
  - For each contract, it checks if the contract address matches the current shard.
  - For each storage update in the contract, it checks if the storage key is in `ptr_to_storage_keys` (using `is_key_in_array`).
  - If the key is relevant and the value actually changed, it is added to `filtered_slots`.

#### 3. Sorting
- After collecting all relevant slots, the code checks if `filtered_slots` is sorted.
- If not, it sorts the array (using `sort_array`).

#### 4. Filtering the Output
- The function `add_crdt_type` is called:
  - It iterates through the original list of storage keys (`ptr_to_storage_keys`).
  - For each key, it checks if it is present in the sorted `filtered_slots`.
  - If yes, it adds the key and its value to the result array.
  - This ensures the output only contains the filtered, relevant state updates, and in the order of the original keys.

#### 5. Merkleization
- The filtered result array is padded with zeros to the next power of two (for Merkle tree requirements).
- The Merkle root is computed over this array (using `merkle_tree_hash`).

#### 6. Output
- The function writes the number of modified contracts and the Merkle root to the output.
- The filtered, sorted, and padded state updates are now ready for on-chain DA.

---

### Summary Table

| Step                | Function(s) Involved         | What Happens?                                                                 |
|---------------------|-----------------------------|-------------------------------------------------------------------------------|
| Collect changes     | `output_contract_state_inner`| Gather only relevant storage keys that changed, per contract                  |
| Filter by keys      | `is_key_in_array`           | Only include keys present in `ptr_to_storage_keys`                            |
| Sort keys           | `is_sorted_recursively`, `sort_array` | Ensure filtered keys are sorted                                      |
| Format output       | `add_crdt_type`             | Output only filtered keys, in original order                                  |
| Pad for Merkle      | `add_zeros`                 | Pad result to next power of two                                               |
| Merkle root         | `merkle_tree_hash`          | Compute Merkle root over filtered, padded output                              |

---

### Key Points
- **Filtering** is done by checking if a key is in the list of interest and if its value actually changed.
- **Order** is preserved by iterating through the original key list when building the output.
- **Merkle root** provides a succinct, tamper-proof summary of the filtered state changes.

---

## Algorithm for Filtering the State Update

### High-Level Goal

The algorithm’s purpose is to:
- **Filter** the contract state changes so that only relevant storage slots (keys) are included in the output.
- **Format** these filtered changes for on-chain data availability (DA).
- **Calculate** a Merkle root over the filtered output for integrity and succinctness.

---

### Step-by-Step Algorithm

#### 1. Preparation
- The function `output_contract_state` is called with:
  - All contract state changes (`contract_state_changes_start`)
  - A list of storage keys of interest (`ptr_to_storage_keys`)
  - Other metadata (number of changes, flags, etc.)

#### 2. Filtering Relevant Slots
- An empty array `filtered_slots` is allocated.
- The function `output_contract_state_inner` is called recursively for each contract:
  - For each contract, it checks if the contract address matches the current shard.
  - For each storage update in the contract, it checks if the storage key is in `ptr_to_storage_keys` (using `is_key_in_array`).
  - If the key is relevant and the value actually changed, it is added to `filtered_slots`.

#### 3. Sorting
- After collecting all relevant slots, the code checks if `filtered_slots` is sorted.
- If not, it sorts the array (using `sort_array`).

#### 4. Filtering the Output
- The function `add_crdt_type` is called:
  - It iterates through the original list of storage keys (`ptr_to_storage_keys`).
  - For each key, it checks if it is present in the sorted `filtered_slots`.
  - If yes, it adds the key and its value to the result array.
  - This ensures the output only contains the filtered, relevant state updates, and in the order of the original keys.

#### 5. Merkleization
- The filtered result array is padded with zeros to the next power of two (for Merkle tree requirements).
- The Merkle root is computed over this array (using `merkle_tree_hash`).

#### 6. Output
- The function writes the number of modified contracts and the Merkle root to the output.
- The filtered, sorted, and padded state updates are now ready for on-chain DA.

---

### Summary Table

| Step                | Function(s) Involved         | What Happens?                                                                 |
|---------------------|-----------------------------|-------------------------------------------------------------------------------|
| Collect changes     | `output_contract_state_inner`| Gather only relevant storage keys that changed, per contract                  |
| Filter by keys      | `is_key_in_array`           | Only include keys present in `ptr_to_storage_keys`                            |
| Sort keys           | `is_sorted_recursively`, `sort_array` | Ensure filtered keys are sorted                                      |
| Format output       | `add_crdt_type`             | Output only filtered keys, in original order                                  |
| Pad for Merkle      | `add_zeros`                 | Pad result to next power of two                                               |
| Merkle root         | `merkle_tree_hash`          | Compute Merkle root over filtered, padded output                              |

---

### Key Points
- **Filtering** is done by checking if a key is in the list of interest and if its value actually changed.
- **Order** is preserved by iterating through the original key list when building the output.
- **Merkle root** provides a succinct, tamper-proof summary of the filtered state changes.
