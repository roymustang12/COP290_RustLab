#include <stdint.h>
#include<stdbool.h>
#include<stdlib.h>
#include<unistd.h>
#include<math.h>
#include "cell.h"
#define INITIAL_CAPACITY 2 // Start with a reasonable size

// struct Cell;
// typedef struct operand
// {
//     int type_flag;
//     //type_flat = 0 for integer constants 
//     //type_flag = 1 for formulas in the cell
//     union {
//         int constant;
//         struct Cell* cell_operand;
//     } operand_value;

// }operand;

// typedef struct  Cell
// {
//     int value;
//     int operation_id;
//     operand (*formula)[];
//     int count_operands;
//     int r;
//     int c;
//     struct Cell** dependents; //This is a pointer to an array of pointers which point to all the cells which  are dependent on it
//     int  count_dependents;
//     struct Cell** precedents;//This is a pointer to an array of pointers which point to all the cells on which the current cell is dependent 
//     int count_precedents;
//     int is_recalculate;
//     bool is_error;
// }Cell;

// Entry states
typedef enum {
    EMPTY = 0,
    OCCUPIED = 1,
    DELETED = 2
} SlotState;

// Hash table entry for storing Cell pointers
typedef struct {
    Cell* cell;     // The cell pointer stored
    SlotState state;  // State of this slot
} HashEntry;

// Hash table structure
typedef struct HashTable{
    HashEntry* entries;  // Array of entries
    int capacity;        // Total size of array
    int count;           // Number of occupied entries
} HashTable;

typedef struct {
    HashTable* table;
    int current_index;
} HashIterator;

unsigned int hash_cell(Cell* cell, int capacity);
HashTable* hash_table_create(void);
void hash_table_resize(HashTable* table);
bool hash_table_insert(HashTable* table, Cell* cell);
bool hash_table_remove(HashTable* table, Cell* cell);
bool hash_table_contains(HashTable* table, Cell* cell);
void hash_table_free(HashTable* table);
HashIterator* hash_iterator_create(HashTable* table);
Cell* hash_iterator_next(HashIterator* iterator);
void hash_iterator_free(HashIterator* iterator);


// Hash function for Cell pointers
unsigned int hash_cell(Cell* cell, int capacity) {
    // Use pointer address as hash value
    uintptr_t addr = (uintptr_t)cell;
    return (unsigned int)(addr % capacity);
}

// Create a new hash table
HashTable* hash_table_create() {
    HashTable* table = (HashTable*)malloc(sizeof(HashTable));
    if (!table) return NULL;
    
    table->capacity = INITIAL_CAPACITY;
    table->count = 0;
    table->entries = (HashEntry*)calloc(table->capacity, sizeof(HashEntry));
    
    if (!table->entries) {
        free(table);
        return NULL;
    }
    
    // Initialize all entries as EMPTY
    for (int i = 0; i < table->capacity; i++) {
        table->entries[i].state = EMPTY;
        table->entries[i].cell = NULL;
    }
    
    return table;
}

// Resize the hash table
void hash_table_resize(HashTable* table) {
    if (!table) return;
    
    int old_capacity = table->capacity;
    HashEntry* old_entries = table->entries;
    
    // Double the size
    table->capacity *= 2;
    table->entries = (HashEntry*)calloc(table->capacity, sizeof(HashEntry));
    if (!table->entries) {
        // Failed to allocate memory, restore old state
        table->capacity = old_capacity;
        table->entries = old_entries;
        return;
    }
    
    // Initialize new entries as EMPTY
    for (int i = 0; i < table->capacity; i++) {
        table->entries[i].state = EMPTY;
        table->entries[i].cell = NULL;
    }
    
    // Reset count and rehash all existing entries
    int old_count = table->count;
    table->count = 0;
    
    for (int i = 0; i < old_capacity; i++) {
        if (old_entries[i].state == OCCUPIED) {
            hash_table_insert(table, old_entries[i].cell);
        }
    }
    
    free(old_entries);
}

// Insert a cell into the hash table
bool hash_table_insert(HashTable* table, Cell* cell) {
    if (!table || !cell) return false;
    
    // Check load factor and resize if needed
    if ((float)table->count / table->capacity > 0.5) {
        hash_table_resize(table);
    }
    
    unsigned int index = hash_cell(cell, table->capacity);
    unsigned int start_index = index;
    
    // Linear probing
    do {
        // If slot is empty or deleted, we can use it
        if (table->entries[index].state != OCCUPIED) {
            table->entries[index].cell = cell;
            table->entries[index].state = OCCUPIED;
            table->count++;
            return true;
        }
        
        // If cell already exists, don't insert again
        if (table->entries[index].state == OCCUPIED && 
            table->entries[index].cell == cell) {
            return false;
        }
        
        // Linear probe to next slot
        index = (index + 1) % table->capacity;
    } while (index != start_index); // Stop if we've gone all the way around
    
    return false; // Table is completely full (shouldn't happen due to resizing)
}

// Remove a cell from the hash table
bool hash_table_remove(HashTable* table, Cell* cell) {
    if (!table || !cell) return false;
    
    unsigned int index = hash_cell(cell, table->capacity);
    unsigned int start_index = index;
    
    // Linear probing to find the cell
    do {
        if (table->entries[index].state == EMPTY) {
            return false; // Reached an empty slot, cell not found
        }
        
        if (table->entries[index].state == OCCUPIED && 
            table->entries[index].cell == cell) {
            // Mark as deleted instead of setting to EMPTY
            // This maintains the probe sequence for lookups
            table->entries[index].state = DELETED;
            table->entries[index].cell = NULL;
            table->count--;
            return true;
        }
        
        // Linear probe to next slot
        index = (index + 1) % table->capacity;
    } while (index != start_index);
    
    return false; // Cell not found
}

// Check if a cell exists in the hash table
bool hash_table_contains(HashTable* table, Cell* cell) {
    if (!table || !cell) return false;
    
    unsigned int index = hash_cell(cell, table->capacity);
    unsigned int start_index = index;
    
    // Linear probing to find the cell
    do {
        if (table->entries[index].state == EMPTY) {
            return false; // Reached an empty slot, cell not found
        }
        
        if (table->entries[index].state == OCCUPIED && 
            table->entries[index].cell == cell) {
            return true; // Found it
        }
        
        // Linear probe to next slot
        index = (index + 1) % table->capacity;
    } while (index != start_index);
    
    return false; // Cell not found after checking all slots
}

// Free all memory used by the hash table
void hash_table_free(HashTable* table) {
    if (!table) return;
    
    free(table->entries);
    free(table);
}

// Iterator for hash table traversal


// Create an iterator for a hash table
HashIterator* hash_iterator_create(HashTable* table) {
    if (!table) return NULL;
    
    HashIterator* iterator = (HashIterator*)malloc(sizeof(HashIterator));
    if (!iterator) return NULL;
    
    iterator->table = table;
    iterator->current_index = -1;
    
    return iterator;
}

// Get next cell from iterator
Cell* hash_iterator_next(HashIterator* iterator) {
    if (!iterator || !iterator->table) return NULL;
    
    // Find next occupied slot
    do {
        iterator->current_index++;
        if (iterator->current_index >= iterator->table->capacity) {
            return NULL; // End of table
        }
    } while (iterator->table->entries[iterator->current_index].state != OCCUPIED);
    
    return iterator->table->entries[iterator->current_index].cell;
}

// Free the iterator
void hash_iterator_free(HashIterator* iterator) {
    free(iterator);
}