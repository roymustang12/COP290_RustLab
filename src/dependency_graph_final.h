#ifndef DEPENDENCY_GRAPH_FINAL_H
#define DEPENDENCY_GRAPH_FINAL_H

#include <stdbool.h>
#include <pthread.h>
#include "cell.h"
#include "hash_table.h"

// Global status variable
extern int status;

// Thread data structure for sleep operations
typedef struct {
    Sheet* sheet;
    int row;
    int col;
    int seconds;
    bool completed;
    pthread_mutex_t mutex;
    pthread_cond_t condition;
} SleepThreadData;

// Core spreadsheet functions
Sheet* initialise(int rows, int columns);
void add_dependency(Sheet* sheet, int rf, int cf, int rt, int ct);
void delete_depedency(Sheet* sheet, int rf, int cf, int rt, int ct);
void clear_precedents(Sheet* sheet, int rt, int ct);
void recalculate_dependents(Sheet* sheet, int r, int c);

// Cycle detection
bool dfs_cycle_detection(Sheet* sheet, Cell* cell, bool* visited, bool* recursion_stack);
bool has_cycle(Sheet* sheet, Cell* start_cell);

// Cell operations
void calculate_cell_value(Sheet* sheet, int rt, int ct);
void assign_cell(Sheet* sheet, int r, int c, int operation_id, operand (*formula)[], int count_operands);
bool precedent_has_error(Sheet* sheet, int r, int c);
bool zero_div_err(Sheet* sheet, int r, int c);

// Formula display
void print_formula(Sheet* sheet, int r, int c);

// Math functions
int min(int a, int b);
int max(int a, int b);
int stdev(int* arr, int n);

// Sleep handling functions
int handle_sleep(Sheet* sheet, int row, int col, int seconds);
void* sleep_thread_function(void* arg);

#endif // DEPENDENCY_GRAPH_FINAL_H