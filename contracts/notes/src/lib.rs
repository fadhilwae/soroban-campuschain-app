#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, 
    Env, String, Symbol, Vec, Address, Map
};

// Struktur data yang akan menyimpan notes
#[contracttype]
#[derive(Clone, Debug)]
pub struct Note {
    id: u64,
    title: String,
    content: String,
    owner: Address,
    created_at: u64,
    updated_at: u64,
    tags: Vec<String>,
}

// Storage keys
const NOTES: Symbol = symbol_short!("NOTES");
const NOTE_COUNT: Symbol = symbol_short!("COUNT");

#[contract]
pub struct NotesContract;

#[contractimpl]
impl NotesContract {
    /// Mendapatkan semua notes milik user tertentu
    pub fn get_notes(env: Env, owner: Address) -> Vec<Note> {
        owner.require_auth();
        
        let all_notes: Map<u64, Note> = env
            .storage()
            .persistent()
            .get(&NOTES)
            .unwrap_or(Map::new(&env));
        
        let mut user_notes = Vec::new(&env);
        
        // Filter notes berdasarkan owner
        for i in 0..all_notes.len() {
            if let Some(key) = all_notes.keys().get(i) {
                if let Some(note) = all_notes.get(key) {
                    if note.owner == owner {
                        user_notes.push_back(note);
                    }
                }
            }
        }
        
        user_notes
    }

    /// Mendapatkan semua notes (untuk admin atau public view)
    pub fn get_all_notes(env: Env) -> Vec<Note> {
        let all_notes: Map<u64, Note> = env
            .storage()
            .persistent()
            .get(&NOTES)
            .unwrap_or(Map::new(&env));
        
        let mut notes_vec = Vec::new(&env);
        
        for i in 0..all_notes.len() {
            if let Some(key) = all_notes.keys().get(i) {
                if let Some(note) = all_notes.get(key) {
                    notes_vec.push_back(note);
                }
            }
        }
        
        notes_vec
    }

    /// Mendapatkan satu note berdasarkan ID
    pub fn get_note(env: Env, id: u64, requester: Address) -> Option<Note> {
        requester.require_auth();
        
        let all_notes: Map<u64, Note> = env
            .storage()
            .persistent()
            .get(&NOTES)
            .unwrap_or(Map::new(&env));
        
        if let Some(note) = all_notes.get(id) {
            // Hanya owner yang bisa akses notenya
            if note.owner == requester {
                return Some(note);
            }
        }
        
        None
    }

    /// Membuat note baru
    pub fn create_note(
        env: Env, 
        owner: Address,
        title: String, 
        content: String,
        tags: Vec<String>
    ) -> u64 {
        owner.require_auth();
        
        // Get atau inisialisasi storage
        let mut all_notes: Map<u64, Note> = env
            .storage()
            .persistent()
            .get(&NOTES)
            .unwrap_or(Map::new(&env));
        
        let mut count: u64 = env
            .storage()
            .persistent()
            .get(&NOTE_COUNT)
            .unwrap_or(0);
        
        // Increment counter untuk ID baru
        count += 1;
        
        let current_timestamp = env.ledger().timestamp();
        
        // Buat note baru
        let note = Note {
            id: count,
            title,
            content,
            owner: owner.clone(),
            created_at: current_timestamp,
            updated_at: current_timestamp,
            tags,
        };
        
        // Simpan note
        all_notes.set(count, note);
        
        // Update storage
        env.storage().persistent().set(&NOTES, &all_notes);
        env.storage().persistent().set(&NOTE_COUNT, &count);
        
        // Extend TTL untuk persistent storage
        env.storage().persistent().extend_ttl(&NOTES, 100, 100);
        env.storage().persistent().extend_ttl(&NOTE_COUNT, 100, 100);
        
        count // Return ID dari note yang baru dibuat
    }

    /// Update note yang sudah ada
    pub fn update_note(
        env: Env,
        id: u64,
        owner: Address,
        title: String,
        content: String,
        tags: Vec<String>
    ) -> bool {
        owner.require_auth();
        
        let mut all_notes: Map<u64, Note> = env
            .storage()
            .persistent()
            .get(&NOTES)
            .unwrap_or(Map::new(&env));
        
        if let Some(mut note) = all_notes.get(id) {
            // Verifikasi owner
            if note.owner != owner {
                return false;
            }
            
            // Update fields
            note.title = title;
            note.content = content;
            note.tags = tags;
            note.updated_at = env.ledger().timestamp();
            
            // Simpan perubahan
            all_notes.set(id, note);
            env.storage().persistent().set(&NOTES, &all_notes);
            env.storage().persistent().extend_ttl(&NOTES, 100, 100);
            
            return true;
        }
        
        false
    }

    /// Menghapus note berdasarkan ID
    pub fn delete_note(env: Env, id: u64, owner: Address) -> bool {
        owner.require_auth();
        
        let mut all_notes: Map<u64, Note> = env
            .storage()
            .persistent()
            .get(&NOTES)
            .unwrap_or(Map::new(&env));
        
        if let Some(note) = all_notes.get(id) {
            // Verifikasi owner
            if note.owner != owner {
                return false;
            }
            
            // Hapus note
            all_notes.remove(id);
            env.storage().persistent().set(&NOTES, &all_notes);
            env.storage().persistent().extend_ttl(&NOTES, 100, 100);
            
            return true;
        }
        
        false
    }

    /// Mencari notes berdasarkan tag
    pub fn get_notes_by_tag(env: Env, owner: Address, tag: String) -> Vec<Note> {
        owner.require_auth();
        
        let all_notes: Map<u64, Note> = env
            .storage()
            .persistent()
            .get(&NOTES)
            .unwrap_or(Map::new(&env));
        
        let mut filtered_notes = Vec::new(&env);
        
        for i in 0..all_notes.len() {
            if let Some(key) = all_notes.keys().get(i) {
                if let Some(note) = all_notes.get(key) {
                    if note.owner == owner {
                        // Cek apakah tag ada di note
                        for j in 0..note.tags.len() {
                            if let Some(note_tag) = note.tags.get(j) {
                                if note_tag == tag {
                                    filtered_notes.push_back(note);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        filtered_notes
    }

    /// Mendapatkan jumlah total notes user
    pub fn get_user_note_count(env: Env, owner: Address) -> u32 {
        owner.require_auth();
        
        let all_notes: Map<u64, Note> = env
            .storage()
            .persistent()
            .get(&NOTES)
            .unwrap_or(Map::new(&env));
        
        let mut count: u32 = 0;
        
        for i in 0..all_notes.len() {
            if let Some(key) = all_notes.keys().get(i) {
                if let Some(note) = all_notes.get(key) {
                    if note.owner == owner {
                        count += 1;
                    }
                }
            }
        }
        
        count
    }
}

mod test;