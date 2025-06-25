use rand::Rng;
use rand::prelude::SliceRandom;
use std::collections::VecDeque;
use std::collections::HashMap;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FragmentInfo {
    pub filename: String,
    pub directory: String,
    pub chunk_indices: Vec<usize>,
}

struct FilenameQueue {
    original: String,
    queue: VecDeque<char>,
}

impl FilenameQueue {
    fn new(name: String) -> Self {
        let queue = name.chars().collect();
        Self {
            original: name,
            queue,
        }
    }
    
    fn pop_front(&mut self) -> Option<char> {
        self.queue.pop_front()
    }
    
    fn front(&self) -> Option<&char> {
        self.queue.front()
    }
    
    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

fn generate_random_filename(max_length: usize) -> String {
    let mut rng = rand::thread_rng();
    
    let length = rng.gen_range(1..=max_length);
    
    let chars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let chars: Vec<char> = chars.chars().collect();
    
    let mut filename = String::new();
    for _ in 0..length {
        let ch = chars[rng.gen_range(0..chars.len())];
        filename.push(ch);
    }
    
    filename
}

fn adjust_char_for_conflict(ch: char) -> char {
    let mut rng = rand::thread_rng();
    let direction = if rng.gen_bool(0.5) { 1 } else { -1 };
    
    let chars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let chars: Vec<char> = chars.chars().collect();
    
    if let Some(current_pos) = chars.iter().position(|&c| c == ch) {
        let new_pos = if direction == 1 {
            (current_pos + 1) % chars.len()
        } else {
            if current_pos == 0 {
                chars.len() - 1
            } else {
                current_pos - 1
            }
        };
        chars[new_pos]
    } else {
        '0'
    }
}

fn resolve_conflicts(queues: &mut [FilenameQueue]) {
    loop {
        let mut conflicts = HashMap::new();
        
        for (i, queue) in queues.iter().enumerate() {
            if let Some(&ch) = queue.front() {
                conflicts.entry(ch).or_insert(Vec::new()).push(i);
            }
        }
        
        let mut has_conflicts = false;
        
        for (_ch, indices) in conflicts {
            if indices.len() > 1 {
                has_conflicts = true;
                for &idx in &indices[1..] {
                    if let Some(front_char) = queues[idx].queue.pop_front() {
                        let new_char = adjust_char_for_conflict(front_char);
                        queues[idx].queue.push_front(new_char);
                    }
                }
            }
        }
        
        if !has_conflicts {
            break;
        }
    }
}

pub fn generate_key_and_fragments(directories: Vec<String>, max_chunks_per_file: usize) -> (String, Vec<FragmentInfo>) {
    let mut rng = rand::thread_rng();
    
    let mut filename_queues: Vec<FilenameQueue> = directories
        .iter()
        .map(|_| {
            let filename = generate_random_filename(max_chunks_per_file);
            FilenameQueue::new(filename)
        })
        .collect();
    
    let mut key = String::new();
    let mut chunk_assignments: Vec<Vec<usize>> = vec![Vec::new(); directories.len()];
    let mut chunk_index = 0;
    
    while filename_queues.iter().any(|q| !q.is_empty()) {
        resolve_conflicts(&mut filename_queues);
        
        let non_empty: Vec<usize> = filename_queues
            .iter()
            .enumerate()
            .filter_map(|(i, q)| if !q.is_empty() { Some(i) } else { None })
            .collect();
        
        if let Some(&selected_idx) = non_empty.choose(&mut rng) {
            if let Some(ch) = filename_queues[selected_idx].pop_front() {
                key.push(ch);
                chunk_assignments[selected_idx].push(chunk_index);
                chunk_index += 1;
            }
        }
    }
    
    let fragments: Vec<FragmentInfo> = directories
        .into_iter()
        .enumerate()
        .zip(filename_queues.iter())
        .map(|((i, dir), queue)| FragmentInfo {
            filename: format!("{}.bin", queue.original),
            directory: dir,
            chunk_indices: chunk_assignments[i].clone(),
        })
        .collect();
    
    (key, fragments)
}

pub fn generate_key(directories: Vec<String>, max_chunks: usize) -> String {
    let (key, _) = generate_key_and_fragments(directories, max_chunks);
    key
}