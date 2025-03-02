use crate::queue::{Task, WorkQueue};
use digest::consts::U32;
use sha2::digest::generic_array::GenericArray;
use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::sync;

pub type Hash = GenericArray<u8, U32>;  // u means unsigned int

#[derive(Debug, Clone)]
pub struct Block {
    pub prev_hash: Hash,      // hash of prev block
    pub generation: u64,      // index of current block (generation 0 has NO prev block)
    pub difficulty: u8,       // amount of work to add block to the chain
    pub data: String,         // actual data in a block
    pub proof: Option<u64>,   
}

impl Block {
    pub fn initial(difficulty: u8) -> Block {
        // TODO: create and return a new initial block
        Block {
            prev_hash: Hash::default(),
            generation: 0,
            difficulty: difficulty,
            data: ("").to_string(),    // cannot write data: "" b/c required type is String but "" is &str (string literal)
            proof: None
        }
    }

    pub fn next(previous: &Block, data: String) -> Block {
        // TODO: create and return a block that could follow `previous` in the chain
        Block {
            prev_hash: previous.hash(),     // get prev block's hash
            generation: previous.generation + 1,
            difficulty: previous.difficulty,
            data: data,
            proof: None
        }
    }

    pub fn hash_string_for_proof(&self, proof: u64) -> String {
        // TODO: return the hash string this block would have if we set the proof to `proof`.
        // self.set_proof(proof);  // borrowing self as immmut => cannot mutate
        format!(
            "{:02x}:{}:{}:{}:{}",
            self.prev_hash,        // Previous hash in hex format
            self.generation,
            self.difficulty,
            self.data,             
            proof                  // Provided proof
        )
    }

    pub fn hash_string(&self) -> String {
        // self.proof.unwrap() panics if block not mined
        let p = self.proof.unwrap();
        self.hash_string_for_proof(p)
    }

    pub fn hash_for_proof(&self, proof: u64) -> Hash {      // implicitly pass ref of this func calling obj as 1st param 
        // TODO: return the block's hash as it would be if we set the proof to `proof`.
        let mut d = Sha256::new();
        d.update(self.hash_string_for_proof(proof));    
        /* 
        must prefix w/ self. as hash_string_for_proof is a method of Block, otherwise Rust assumes this func is local func
        defined in same scope of current method
        */  
        // dont have to pass self as param 
        d.finalize()
    }

    pub fn hash(&self) -> Hash {
        // self.proof.unwrap() panics if block not mined
        let p = self.proof.unwrap();
        self.hash_for_proof(p)
    }

    pub fn set_proof(self: &mut Block, proof: u64) {
        self.proof = Some(proof);
    }

    pub fn hash_satisfies_difficulty(difficulty:u8,hash:Hash) -> bool {
        // TODO: does the hash `hash` have `difficulty` trailing 0s
        if difficulty == 0 {
            return true;
        }
        
        let n_bytes = difficulty / 8;
        let n_bits = difficulty % 8;

        // check if last n_bytes ele satisfy
        for i in 0..n_bytes {
            if hash[hash.len() - 1 - i as usize] != 0u8 {
                return false;
            }
        }

        for i in 0..n_bits {
            if (1 << n_bits) & (hash[hash.len() - 1 - i as usize]) != 0 {
                return false;
            }
        }


        // DIFF APPROACH
        // let n_bytes = (difficulty / 8) as usize;
        // let n_bits = difficulty % 8;

        // // check if last n_bytes ele satisfy
        // for i in 0..n_bytes {
        //     if hash[hash.len() - 1 - i] != 0u8 {
        //         return false;
        //     }
        // }

        // if n_bits > 0 {
        //     let n = (hash[31 - n_bytes]) as usize;
        //     if n % (1 << n_bits) != 0 {
        //         return false;
        //     }           
        // }

        true
    }

    pub fn is_valid_for_proof(&self, proof: u64) -> bool {
        Self::hash_satisfies_difficulty(self.difficulty,self.hash_for_proof(proof))
    }

    pub fn is_valid(&self) -> bool {
        if self.proof.is_none() {
            return false;
        }
        self.is_valid_for_proof(self.proof.unwrap())
    }

    // Mine in a very simple way: check sequentially until a valid hash is found.
    // This doesn't *need* to be used in any way, but could be used to do some mining
    // before your .mine is complete. Results should be the same as .mine (but slower).
    pub fn mine_serial(self: &mut Block) {
        let mut p = 0u64;
        while !self.is_valid_for_proof(p) {
            p += 1;
        }
        self.proof = Some(p);
    }

    pub fn mine_range(self: &Block, workers: usize, start: u64, end: u64, chunks: u64) -> u64 {
        // TODO: with `workers` threads, check proof values in the given range, breaking up
	    // into `chunks` tasks in a work queue. Return the first valid proof found.
        // HINTS:
        // - Create and use a queue::WorkQueue.
        // - Use sync::Arc to wrap a clone of self for sharing.
            
        // Create a work queue with the specified number of workers
        let mut queue = WorkQueue::new(workers);
                
        // Create an Arc<Block> for sharing across threads
        let block = sync::Arc::new(self.clone());

        // Calculate the size of each chunk
        let chunk_size = ((end - start) + chunks - 1) / chunks;

        // Create and submit tasks for each chunk
        for chunk_idx in 0..chunks {
            let chunk_start = start + chunk_idx * chunk_size;
            let chunk_end = if chunk_idx == chunks - 1 {
                // Make sure the last chunk includes any remaining values
                end
            } else {
                chunk_start.min(end).saturating_add(chunk_size).min(end)
            };

            // Not create empty chunks
            if chunk_start >= end || chunk_start >= chunk_end {
                continue;
            }
            
            // Create a new mining task for this chunk
            let task = MiningTask {
                block: block.clone(),
                start: chunk_start,
                end: chunk_end,
            };
            
            // Add the task to the work queue
            let _ = queue.enqueue(task);
        }

        let tasks_submitted = chunks.min((end - start + chunk_size - 1) / chunk_size);

        for _ in 0..tasks_submitted {
            let proof = queue.recv();
            if proof < end {  // Valid proof found
                return proof;
            }
        }
        // If no valid proof was found, return the end value
        end

    }

    pub fn mine_for_proof(self: &Block, workers: usize) -> u64 {
        let range_start: u64 = 0;
        let range_end: u64 = 8 * (1 << self.difficulty); // 8 * 2^(bits that must be zero)
        let chunks: u64 = 2345;
        self.mine_range(workers, range_start, range_end, chunks)
    }

    pub fn mine(self: &mut Block, workers: usize) {
        self.proof = Some(self.mine_for_proof(workers));
    }
}

struct MiningTask {
    block: sync::Arc<Block>,
    // TODO: more fields as needed
    start: u64,
    end: u64
}

impl Task for MiningTask {
    type Output = u64;

    fn run(&self) -> Option<u64> {
        // TODO: what does it mean to .run?
        
        // Loop thru range of proofs assigned to this task
        for proof in self.start..self.end {
            if self.block.is_valid_for_proof(proof) {   // check proofs
                return Some(proof);
            }
        }
        None
    }
}