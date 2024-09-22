// Inspired by https://github.com/kishimisu/WebGPU-Radix-Sort/blob/4c7d5cb8cf6bc584b18ee3ee38422b74e52fb6b3/src/shaders/prefix_sum.js

#import types::WorldSettings;

@group(0) @binding(0) var<uniform> settings: WorldSettings;
@group(0) @binding(1) var<storage, read_write> items: array<u32>;
@group(0) @binding(2) var<storage, read_write> blockSums: array<u32>;

const WORKGROUP_SIZE_X: u32 = 32;
const WORKGROUP_SIZE_Y: u32 = 32;
const THREADS_PER_WORKGROUP: u32 = WORKGROUP_SIZE_X * WORKGROUP_SIZE_Y;
const ITEMS_PER_WORKGROUP: u32 = THREADS_PER_WORKGROUP * 2;

const WORKGROUP_MEMORY_SIZE: u32 = ITEMS_PER_WORKGROUP * 2;
var<workgroup> temp: array<u32, WORKGROUP_MEMORY_SIZE>;

@compute @workgroup_size(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, 1)
fn reduce_downsweep(
    @builtin(workgroup_id) w_id: vec3<u32>,
    @builtin(num_workgroups) w_dim: vec3<u32>,
    @builtin(local_invocation_index) TID: u32, // Local thread ID
) {
    let TOTAL_ITEMS = (settings.grid_dimensions.x * settings.grid_dimensions.y) + 2;
    let WORKGROUP_ID = w_id.x + w_id.y * w_dim.x;
    let WID = WORKGROUP_ID * THREADS_PER_WORKGROUP;
    let GID = WID + TID; // Global thread ID

    let ELM_TID = TID * 2; // Element pair local ID
    let ELM_GID = (GID * 2); // Element pair global ID

    // Load input to shared memory
    if ELM_GID >= TOTAL_ITEMS {
        temp[ELM_TID] = 0u;
    } else {
        temp[ELM_TID] = items[ELM_GID];
    }
    if ELM_GID + 1 >= TOTAL_ITEMS {
        temp[ELM_TID + 1] = 0u;
    } else {
        temp[ELM_TID + 1] = items[ELM_GID + 1];
    }

    var offset: u32 = 1;

    // Up-sweep (reduce) phase
    for (var d: u32 = ITEMS_PER_WORKGROUP >> 1; d > 0; d = d >> 1) {
        workgroupBarrier();

        if TID < d {
            var ai: u32 = offset * (ELM_TID + 1) - 1;
            var bi: u32 = offset * (ELM_TID + 2) - 1;
            temp[bi] += temp[ai];
        }

        offset = offset * 2;
    }

    // Save workgroup sum and clear last element
    if TID == 0 {
        let last_offset = ITEMS_PER_WORKGROUP - 1;

        blockSums[WORKGROUP_ID] = temp[last_offset];
        temp[last_offset] = 0u;
    }

    // Down-sweep phase
    for (var d: u32 = 1; d < ITEMS_PER_WORKGROUP; d = d * 2) {
        offset = offset >> 1;
        workgroupBarrier();

        if TID < d {
            var ai: u32 = offset * (ELM_TID + 1) - 1;
            var bi: u32 = offset * (ELM_TID + 2) - 1;

            let t: u32 = temp[ai];
            temp[ai] = temp[bi];
            temp[bi] += t;
        }
    }
    workgroupBarrier();

    // @tombh: ...
    let final_index = ELM_GID;

    // Copy result from shared memory to global memory
    if final_index >= TOTAL_ITEMS {
        return;
    }
    items[final_index] = temp[ELM_TID];

    if final_index + 1 >= TOTAL_ITEMS {
        return;
    }
    items[final_index + 1] = temp[ELM_TID + 1];
}

@compute @workgroup_size(WORKGROUP_SIZE_X, WORKGROUP_SIZE_Y, 1)
fn add_block_sums(
    @builtin(workgroup_id) w_id: vec3<u32>,
    @builtin(num_workgroups) w_dim: vec3<u32>,
    @builtin(local_invocation_index) TID: u32, // Local thread ID
) {
    let TOTAL_ITEMS = (settings.grid_dimensions.x * settings.grid_dimensions.y) + 2;
    let WORKGROUP_ID = w_id.x + w_id.y * w_dim.x;
    let WID = WORKGROUP_ID * THREADS_PER_WORKGROUP;
    let GID = WID + TID; // Global thread ID

    let ELM_ID = GID * 2;

    // @tombh: ...
    // let final_index = ELM_ID - 1;

    if ELM_ID >= TOTAL_ITEMS {
        return;
    }
    let blockSum = blockSums[WORKGROUP_ID];
    items[ELM_ID] += blockSum;

    if ELM_ID + 1 >= TOTAL_ITEMS {
        return;
    }
    items[ELM_ID + 1] += blockSum;
}
