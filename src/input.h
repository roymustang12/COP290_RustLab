#ifndef INPUT_H
#define INPUT_H

#include <stdbool.h>
#include "dependency_graph_final.h"  // for Sheet and operand definitions

// Global variables declared in input.c
extern int operationID;
extern int editrow;
extern int editcolumn;
extern int count_operands;
extern operand (*formula)[];

// Function prototypes
void parseCellName(const char* cellName, int* row, int* col);
void parseInput(const char* input, Sheet* spreadsheet, int rows, int cols);
int isArithmeticExpression(const char* expression);
int isFunction(const char* expression);
int AssignValue(char* op);
int string_to_int(const char *num_str);
int count_occurrences(char ch, const char *str) ;

#endif // INPUT_H