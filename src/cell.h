#ifndef CELL_H
#define CELL_H

#include <stdbool.h>

// Forward declaration for HashTable
struct HashTable;
typedef struct HashTable HashTable;

// Operand structure definition
typedef struct operand {
    int type_flag;
    //type_flag = 0 for integer constants 
    //type_flag = 1 for formulas in the cell
    union {
        int constant;
        struct Cell* cell_operand;
    } operand_value;
} operand;

// Cell structure definition
typedef struct Cell {
    int value;
    int operation_id;
    operand (*formula)[];
    int count_operands;
    int r;
    int c;
    HashTable* dependents;
    int count_dependents; // Cells dependent on this cell
    HashTable* precedents; 
    int count_precedents;  // Cells this cell depends on
    int is_recalculate;
    bool is_error;
} Cell;

// Define Spreadsheet structure
typedef struct Spreadsheet {
    int rows;
    int columns;
    Cell*** all_cells;
} Sheet;

#endif // CELL_H