#ifndef HASH_TABLE_H
#define HASH_TABLE_H

#include <stdbool.h>
#include<stdlib.h>
#include<unistd.h>
#include <stdint.h>
#include<math.h>
#include "cell.h"

// // Forward declarations
// struct Cell;
// typedef struct Cell Cell;

// Entry states for hash table slots
typedef enum {
    EMPTY = 0,
    OCCUPIED = 1,
    DELETED = 2
} SlotState;

// Hash table entry for storing Cell pointers
typedef struct {
    Cell* cell;         // The cell pointer stored
    SlotState state;    // State of this slot
} HashEntry;

// Hash table structure
typedef struct HashTable{
    HashEntry* entries; // Array of entries
    int capacity;       // Total size of array
    int count;          // Number of occupied entries
} HashTable;

// Iterator for hash table traversal
typedef struct {
    HashTable* table;
    int current_index;
} HashIterator;

// Hash table functions
HashTable* hash_table_create();
void hash_table_resize(HashTable* table);
bool hash_table_insert(HashTable* table, Cell* cell);
bool hash_table_remove(HashTable* table, Cell* cell);
bool hash_table_contains(HashTable* table, Cell* cell);
void hash_table_free(HashTable* table);

// Hash table iterator functions
HashIterator* hash_iterator_create(HashTable* table);
Cell* hash_iterator_next(HashIterator* iterator);
void hash_iterator_free(HashIterator* iterator);

#endif // HASH_TABLE_H