#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <time.h>
#include<math.h>
#include <stdbool.h>
#include<pthread.h>
#include "cell.h"
#include "hash_table.h"


int status = 0;
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

// typedef struct Coord
// {
//     int x;
//     int y;
// };

//Sheet contains a 2D array of Cell pointers 
//all_cell is a pointer to the sheet
//all_cells contain a 
// typedef struct Spreadsheet
// {
//     int rows;
//     int columns;
//     Cell*** all_cells; //pointer to an array of pointers which point to an array of pointers which refer to Cell
// }Sheet;

typedef struct {
    Sheet* sheet;
    int row;
    int col;
    int seconds;
    bool completed;
    pthread_mutex_t mutex;
    pthread_cond_t condition;
} SleepThreadData;

Sheet* initialise(int rows, int columns);
void add_dependency(Sheet* sheet, int rf, int cf, int rt, int ct);
void delete_depedency(Sheet* sheet, int rf, int cf, int rt, int ct);
void clear_precedents(Sheet* sheet, int rt, int ct);
void recalculate_dependents(Sheet* sheet, int r, int c);
// void recalculate_dependents_2(Sheet* sheet, int r, int c);
void topological_sort(Sheet* sheet, Cell* start_cell);
void dfs_topological_sort(Sheet* sheet, Cell* cell, Cell** stack, int* stack_size, bool* visited);
bool dfs_cycle_detection(Sheet* sheet,Cell* cell, bool* visited, bool* recursion_stack);
int detect_cycle(Cell* cell, Cell** visited, Cell** recursion_stack, int* visited_count, int* stack_count);
bool has_cycle(Sheet* sheet, Cell* start_cell);
void calculate_cell_value(Sheet* sheet, int rt, int ct);
void assign_cell(Sheet* sheet, int r, int c, int operation_id, operand (*formula)[], int count_operands);
int min(int a, int b);
int max(int a, int b);
void print_formula(Sheet* sheet, int r, int c);
bool precedent_has_error(Sheet* sheet, int r, int c);
bool zero_div_err(Sheet* sheet, int r, int c);
int handle_sleep(Sheet* sheet, int row, int col,int seconds);
int stdev(int* arr, int n);
void* sleep_thread_function(void* arg);

Sheet* initialise(int rows, int columns)
{
    Sheet* sheet = (Sheet*)malloc(sizeof(Sheet));
    sheet->rows = rows;
    sheet->columns = columns;
    sheet->all_cells = (Cell***)malloc(sizeof(Cell**) * rows);
    clock_t start_init = clock();
    for(int i = 0; i < sheet->rows; i++)
    {
        sheet->all_cells[i] = (Cell**)malloc(sizeof(Cell*) * columns);
        for(int j = 0;j < sheet->columns; j++)
        {
           sheet->all_cells[i][j] = (Cell*)malloc(sizeof(Cell));
           sheet->all_cells[i][j]->r = i;
           sheet->all_cells[i][j]->c = j;
           sheet->all_cells[i][j]->value = 0;
           sheet->all_cells[i][j]->operation_id = -1; //operand_id = -1 when cell is not assigned any formula
           sheet->all_cells[i][j]->count_dependents = 0;
           sheet->all_cells[i][j]->count_precedents = 0;
           sheet->all_cells[i][j]->formula = NULL; // Assuming a maximum of 10 operands for simplicity
           sheet->all_cells[i][j]->dependents = hash_table_create();
           sheet->all_cells[i][j]->precedents = hash_table_create();
           sheet->all_cells[i][j]->is_recalculate = 0;
           sheet->all_cells[i][j]->count_operands = 0;
           sheet->all_cells[i][j]->is_error = false;
            //0 when no recalculation is needed 
        }
    }
    clock_t init_end = clock();
    double init_time = (init_end - start_init) / CLOCKS_PER_SEC;
    printf("INIT_TIME: %f", init_time);
    return sheet;
}

//Cell rt,ct contains the cell rf,cf in the formula expresssion
//Add rt,ct to the dependents of rf,cf
//Add rf,cf to the precedents of rt,ct
void add_dependency(Sheet* sheet, int rf, int cf, int rt, int ct)
{
    Cell* dependent_cell = sheet->all_cells[rt][ct];
    Cell* precedent_cell = sheet->all_cells[rf][cf];
    precedent_cell->count_dependents+=1;
    // precedent_cell->dependents = realloc(precedent_cell->dependents,sizeof(Cell*) * precedent_cell->count_dependents);
    // precedent_cell->dependents[precedent_cell->count_dependents - 1] = dependent_cell;
    hash_table_insert(precedent_cell->dependents, dependent_cell);
    dependent_cell->count_precedents+=1;
    // dependent_cell->precedents = realloc(dependent_cell->precedents,sizeof(Cell*) * dependent_cell->count_precedents);
    // dependent_cell->precedents[dependent_cell->count_precedents - 1] = precedent_cell;
    hash_table_insert(dependent_cell->precedents, precedent_cell);
}

//Cell rt,ct contains the cell rf,cf in the formula expresssion
//When updating the formula of rt,ct we need to remove the cell rt,ct 
//list of depedents of rf,cf 
void delete_depedency(Sheet* sheet, int rf, int cf, int rt, int ct)
{
    Cell* dependent_cell = sheet->all_cells[rt][ct];
    Cell* precedent_cell = sheet->all_cells[rf][cf];
    hash_table_remove(precedent_cell->dependents, dependent_cell);
    // int target_index = -1;
    // for(int i = 0; i < precedent_cell->count_dependents; i++)
    // {
    //     if(precedent_cell->dependents[i]->r == rt && precedent_cell->dependents[i]->c == ct)
    //     {
    //         target_index = i;
    //         break;
    //     }
    // }
    // //Ab yaha se memory hata and resize
    // if (target_index != -1)
    // {
    //     for(int i = target_index; i < target_index - 1; i++)
    //     {
    //         precedent_cell->dependents[i] = precedent_cell->dependents[i+1];
    //     }
    //     precedent_cell->count_dependents-=1;
    //     if(precedent_cell->count_dependents > 0)
    //     {
    //         precedent_cell->dependents = realloc(precedent_cell->dependents, sizeof(Cell*) * precedent_cell->count_dependents);
    //     }
    //     else
    //     {
    //         free(precedent_cell->dependents);
    //         precedent_cell->dependents = NULL;
    //     }
    // }
} 

//When a cell is assigned an expression we need to clear all the old precedents of this cell
void clear_precedents(Sheet* sheet, int rt, int ct)
{
    Cell* target_cell = sheet->all_cells[rt][ct];
    // free(target_cell->precedents);
    hash_table_free(target_cell->precedents);
    // target_cell->precedents = NULL;
    target_cell->count_precedents = 0;
    target_cell->precedents = hash_table_create();
}

// On updation of value of cell at r,c we need to update all
// the cells dependent on the cell recently assigned 
void recalculate_dependents(Sheet* sheet, int r, int c)
{   /*if (has_cycle(sheet, sheet->all_cells[r][c]) == false)*/
        {Cell** q = (Cell**)malloc(sizeof(Cell*) * sheet->rows * sheet->columns);
        int front = 0;
        int rear = 0;
        Cell* assigned_cell = sheet->all_cells[r][c];
        //Add the immediate dependents of the assigned cell to the queue
        // for(int i = 0; i < assigned_cell->count_dependents; i++)
        // {
        //     q[rear++] = (assigned_cell->dependents)[i];

        // }
        HashIterator* iterator = hash_iterator_create(assigned_cell->dependents);
        Cell* dependent;
        while ((dependent = hash_iterator_next(iterator)) != NULL) {
            q[rear++] = dependent;
        }
        hash_iterator_free(iterator);

        while(front < rear)
        {
            Cell* current_cell = q[front];
            front = front + 1;
            if (has_cycle(sheet, current_cell) == false)
            {
                if(zero_div_err(sheet, current_cell->r, current_cell->c) == true)
                {
                    // printf("--ZERO DIV ERROR STARTS\n");
                    // printf("--ZERO DIV ERRRO END");
                    status = 2;
                    current_cell->is_error = true;
                }
                 else if (precedent_has_error(sheet, current_cell->r, current_cell->c) == true)
                 {
                    // printf("--PRECEDENT ME ERROR\n");
                    // printf("--PRECEDENT ME ERROR END");
                    status = 2;
                    current_cell->is_error = true;
                }
                else{
                    calculate_cell_value(sheet, current_cell->r, current_cell->c);}
                
            }
            //Add the dependents of the current_cell to the queue
            // for(int i = 0; i < current_cell->count_dependents;i++)
            // {
            //     q[rear++] = current_cell->dependents[i];
            // }
            iterator = hash_iterator_create(current_cell->dependents);
            while ((dependent = hash_iterator_next(iterator)) != NULL) {
                q[rear++] = dependent;
            }
            hash_iterator_free(iterator);
        }

        free(q);}
    // else{
    //     printf("CHUD GAYE");
    //     status = 3;
    // }
}

// void recalculate_dependents_2(Sheet* sheet, int r, int c) {
//     if (has_cycle(sheet, sheet->all_cells[r][c]) == 0) {
//         topological_sort(sheet, sheet->all_cells[r][c]);
//     } else {
//         status = 17; // Cycle detected
//     }
// }

// void topological_sort(Sheet* sheet, Cell* start_cell) {
//     Cell** stack = (Cell**)malloc(sizeof(Cell*) * sheet->rows * sheet->columns);
//     int stack_size = 0;
//     bool* visited = (bool*)calloc(sheet->rows * sheet->columns, sizeof(bool));

//     dfs_topological_sort(sheet, start_cell, stack, &stack_size, visited);

//     // Recalculate cells in topological order
//     while (stack_size > 0) {
//         Cell* cell = stack[--stack_size];
//         calculate_cell_value(sheet, cell->r, cell->c);
//     }

//     free(stack);
//     free(visited);
// }

// void dfs_topological_sort(Sheet* sheet, Cell* cell, Cell** stack, int* stack_size, bool* visited) {
//     int cell_index = cell->r * sheet->columns + cell->c;
//     visited[cell_index] = true;

//     for (int i = 0; i < cell->count_dependents; i++) {
//         Cell* dependent = cell->dependents[i];
//         int dependent_index = dependent->r * sheet->columns + dependent->c;
//         if (!visited[dependent_index]) {
//             dfs_topological_sort(sheet, dependent, stack, stack_size, visited);
//         }
//     }

//     stack[(*stack_size)++] = cell;
// }


// int detect_cycle(Cell* cell, Cell** visited, Cell** recursion_stack, int* visited_count, int* stack_count) {
//     // Add current cell to visited and recursion stack
//     visited[*visited_count] = cell;
//     (*visited_count)++;
//     recursion_stack[*stack_count] = cell;
//     (*stack_count)++;
    
//     // Check all cells this cell depends on (precedents)
//     for (int i = 0; i < cell->count_precedents; i++) {
//         Cell* precedent = cell->precedents[i];
        
//         // If the precedent is in the recursion stack, a cycle is found
//         for (int j = 0; j < *stack_count; j++) {
//             if (precedent == recursion_stack[j]) {
//                 return 1;
//             }
//         }
        
//         // Check if the precedent has already been visited
//         int already_visited = 0;
//         for (int j = 0; j < *visited_count; j++) {
//             if (precedent == visited[j]) {
//                 already_visited = 1;
//                 break;
//             }
//         }
        
//         // If not visited, recursively check for cycles
//         if (!already_visited) {
//             if (detect_cycle(precedent, visited, recursion_stack, visited_count, stack_count)) {
//                 return 1;
//             }
//         }
//     }
    
//     // Remove current cell from recursion stack
//     (*stack_count)--;
//     return 0;
// }

// int has_cycle(Sheet* sheet, Cell* start_cell) {
//     int max_cells = sheet->rows * sheet->columns;
//     Cell** visited = malloc(max_cells * sizeof(Cell*));
//     Cell** recursion_stack = malloc(max_cells * sizeof(Cell*));
//     int visited_count = 0;
//     int stack_count = 0;
    
//     int result = detect_cycle(start_cell, visited, recursion_stack, &visited_count, &stack_count);
    
//     free(visited);
//     free(recursion_stack);
//     return result;
// }

bool dfs_cycle_detection(Sheet* sheet,Cell* cell, bool* visited, bool* recursion_stack) {
    int cell_index = cell->r * sheet->columns + cell->c;

    if (recursion_stack[cell_index]) return true; // Cycle detected
    if (visited[cell_index]) return false; // Already processed

    visited[cell_index] = true;
    recursion_stack[cell_index] = true;
    HashIterator* iterator = hash_iterator_create(cell->dependents);
    Cell* dependent;
    // for (int i = 0; i < cell->count_dependents; i++) {
    //     if (dfs_cycle_detection(sheet, cell->dependents[i], visited, recursion_stack)) {
    //         return true;
    //     }
    // }
    bool cycle_found = false;

    while ((dependent = hash_iterator_next(iterator)) != NULL) {
        if (dfs_cycle_detection(sheet, dependent, visited, recursion_stack)) {
            cycle_found = true;
            // hash_iterator_free(iterator);
            // return true;
            break;
        }
    }
    hash_iterator_free(iterator);
    recursion_stack[cell_index] = false;
    return cycle_found;
}

bool has_cycle(Sheet* sheet, Cell* start_cell) {
    int total_cells = sheet->rows * sheet->columns;
    bool* visited = (bool*)calloc(total_cells, sizeof(bool));
    bool* recursion_stack = (bool*)calloc(total_cells, sizeof(bool));

    bool cycle_found = dfs_cycle_detection(sheet, start_cell, visited, recursion_stack);

    free(visited);
    free(recursion_stack);
    return cycle_found;
}

void calculate_cell_value(Sheet* sheet, int rt, int ct){
    Cell* target_cell = sheet->all_cells[rt][ct];
    switch(target_cell->operation_id)
    {
        case 1 :
                {int value = (*(target_cell->formula))[0].operand_value.constant;
                target_cell->value = value;
                target_cell->is_error = false;
                status = 0;
                break;}
        case 2 :
                {int value = (*(target_cell->formula))[0].operand_value.cell_operand->value;
                bool error = (*(target_cell->formula))[0].operand_value.cell_operand->is_error;
                
                if(error == false)
                {
                    target_cell->value = value;
                    status = 0;
                    target_cell->is_error = false;
                }
                else
                {
                    target_cell->is_error = true;
                    status = 2;
                }
                break;}
        case 3:
                {int flag1 = (*(target_cell->formula))[0].type_flag;
                int flag2 = (*(target_cell->formula))[1].type_flag;
                int value1, value2;
                bool err1 = false;
                bool err2 = false;
                if (flag1 == 0)
                {
                    value1 = (*(target_cell->formula))[0].operand_value.constant;
                }
                else
                {
                    value1 = (*(target_cell->formula))[0].operand_value.cell_operand->value;
                    err1  = (*(target_cell->formula))[0].operand_value.cell_operand->is_error;
                }
                if (flag2 == 0)
                {
                    value2 = (*(target_cell->formula))[1].operand_value.constant;
                }
                else
                {
                    value2 = (*(target_cell->formula))[1].operand_value.cell_operand->value;
                    err2 = (*(target_cell->formula))[1].operand_value.cell_operand->is_error;
                }
                if(err1 == false && err2 == false)
                    {
                        target_cell->value = value1 + value2;
                        status = 0;
                        target_cell->is_error = false;

                    }
                else
                {
                    target_cell->is_error = true;
                    status = 2;
                }
                break;}
        case 4:
                {int flag1 = (*(target_cell->formula))[0].type_flag;
                int flag2 = (*(target_cell->formula))[1].type_flag;
                int value1, value2;
                bool err1= false;
                bool err2 = false;
                if (flag1 == 0)
                {
                    value1 = (*(target_cell->formula))[0].operand_value.constant;
                }
                else
                {
                    value1 = (*(target_cell->formula))[0].operand_value.cell_operand->value;
                    err1  = (*(target_cell->formula))[0].operand_value.cell_operand->is_error;

                }
                if (flag2 == 0)
                {
                    value2 = (*(target_cell->formula))[1].operand_value.constant;
                }
                else
                {
                    value2 = (*(target_cell->formula))[1].operand_value.cell_operand->value;
                    err2  = (*(target_cell->formula))[1].operand_value.cell_operand->is_error;

                }
                if(err1 == false && err2 == false)
                    {
                        target_cell->value = value1 - value2;
                        status = 0;
                        target_cell->is_error = false;
                    }
                else
                {
                    target_cell->is_error = true;
                    status = 2;
                }
                break;}
    case 5 :
                {int flag1 = (*(target_cell->formula))[0].type_flag;
                int flag2 = (*(target_cell->formula))[1].type_flag;
                int value1, value2;
                bool err1 = false;
                bool err2 = false;
                if (flag1 == 0)
                {
                    value1 = (*(target_cell->formula))[0].operand_value.constant;
                }
                else
                {
                    value1 = (*(target_cell->formula))[0].operand_value.cell_operand->value;
                    err1  = (*(target_cell->formula))[0].operand_value.cell_operand->is_error;

                }
                if (flag2 == 0)
                {
                    value2 = (*(target_cell->formula))[1].operand_value.constant;
                }
                else
                {
                    value2 = (*(target_cell->formula))[1].operand_value.cell_operand->value;
                    err2  = (*(target_cell->formula))[1].operand_value.cell_operand->is_error;

                }
                if(err1 == false && err2 == false)
                    {
                        target_cell->value = value1 * value2;
                        status = 0;
                        target_cell->is_error = false;
                    }
                else
                {
                    target_cell->is_error = true;
                    status = 2;
                }
                break;}
    case 6 :
                {int flag1 = (*(target_cell->formula))[0].type_flag;
                int flag2 = (*(target_cell->formula))[1].type_flag;
                int value1, value2;
                bool err1= false;
                bool err2  =false;
                if (flag1 == 0)
                {
                    value1 = (*(target_cell->formula))[0].operand_value.constant;
                }
                else
                {
                    value1 = (*(target_cell->formula))[0].operand_value.cell_operand->value;
                    err1  = (*(target_cell->formula))[0].operand_value.cell_operand->is_error;

                }
                if (flag2 == 0)
                {
                    value2 = (*(target_cell->formula))[1].operand_value.constant;
                }
                else
                {
                    value2 = (*(target_cell->formula))[1].operand_value.cell_operand->value;
                    err2  = (*(target_cell->formula))[1].operand_value.cell_operand->is_error;

                }
                if(value2 == 0)
                {
                    target_cell->is_error = true;
                    status = 2;
                }
                else if(err1==true || err1==true)
                {
                    target_cell->is_error = true;
                    status = 2;
                }
                else
                    {
                        target_cell->value = value1 / value2;
                        status = 0;
                        target_cell->is_error = false;
                    }
                break;}
    case 7 :
                {int values[target_cell->count_operands];
                bool err[target_cell->count_operands];
                int flag;
                for(int i = 0; i < target_cell->count_operands ; i++)
                {
                    flag = (*(target_cell->formula))[i].type_flag;
                    if (flag == 0)
                    {
                        values[i] = (*(target_cell->formula))[i].operand_value.constant;
                    }
                    else
                    {
                        values[i] = (*(target_cell->formula))[i].operand_value.cell_operand->value;
                        err[i] = (*(target_cell->formula))[i].operand_value.cell_operand->is_error;
                    }
                }
                int temp = values[0];
                bool exit_case = false;
                for(int i = 0; i < target_cell->count_operands; i++)
                {
                    temp = min(temp, values[i]);
                    if(err[i]==true)
                    {
                        target_cell->is_error = true;
                        status = 2;
                        exit_case = true;
                        break;
                    }
                }
                if(exit_case)
                {
                    break;
                }
                target_cell->value = temp;
                status = 0;
                target_cell->is_error = false;
                break;}
        case 8 : 
                {
                // printf("COUNT OPERANDS : %d\n",target_cell->count_operands);
                int values[target_cell->count_operands];
                bool err[target_cell->count_operands];
                int flag;
                for(int i = 0; i < target_cell->count_operands ; i++)
                {
                    // printf("MAKING VALUES");
                    flag = (*(target_cell->formula))[i].type_flag;
                    if (flag == 0)
                    {
                        values[i] = (*(target_cell->formula))[i].operand_value.constant;
                    }
                    else
                    {
                        values[i] = (*(target_cell->formula))[i].operand_value.cell_operand->value;
                        err[i] = (*(target_cell->formula))[i].operand_value.cell_operand->is_error;
                        // printf("INDIVIDUAL ERROR VALUES");
                        // printf("%d\n",err[i]);
                        // printf("---INDIVIDUAL ERROR VALUES ENDS");
                    }
                }
                int temp = values[0];
                bool exit_case = false;
                for(int i = 0; i < target_cell->count_operands; i++)
                {
                    // printf("MAKING TEMP");
                   temp = max(temp, values[i]);
                   if(err[i] == true)
                   {
                    // printf("ERROR DETECTED");
                    target_cell->is_error = true;
                    status = 2;
                    exit_case = true;
                    break;
                   }
                }
                if(exit_case == true)
                {
                    break;
                }
                // printf("----CELL MAX VALUE----\n");
                // printf("%d\n",temp);
                // printf("----CELL MAX VALUE END--\n");
                // printf("COUNT OPERANDS : %d\n",target_cell->count_operands);
                // for(int i = 0; i < target_cell->count_operands;i++)
                // {
                //     printf("%d\n",err[i]);
                // }
                target_cell->value = temp;
                status = 0;
                target_cell->is_error = false;
                break;   
                }
        case 9:
                {int values[target_cell->count_operands];
                    bool err[target_cell->count_operands];
                int flag;
                for(int i = 0; i < target_cell->count_operands ; i++)
                {
                    flag = (*(target_cell->formula))[i].type_flag;
                    if (flag == 0)
                    {
                        values[i] = (*(target_cell->formula))[i].operand_value.constant;
                    }
                    else
                    {
                        values[i] = (*(target_cell->formula))[i].operand_value.cell_operand->value;
                        err[i] = (*(target_cell->formula))[i].operand_value.cell_operand->is_error;
                    }
                }
                int temp = 0;
                bool exit_case = false;
                for(int i = 0; i < target_cell->count_operands; i++)
                {
                   temp = temp + values[i];
                   if(err[i]==true)
                   {
                    target_cell->is_error = true;
                    status = 2;
                    exit_case = true;
                    break;
                   }
                }
                if(exit_case)
                {
                    break;
                }
                // printf("----TEMP VALUE AVG\n");
                // printf("%d\n",temp);
                // printf("----TEMP VALUE AVG END--\n");
                target_cell->value = temp/(target_cell->count_operands);
                status = 0;
                target_cell->is_error = false;
                break;}
        case 10 :
                {
                    int values[target_cell->count_operands];
                    bool err[target_cell->count_operands];
                int flag;
                for(int i = 0; i < target_cell->count_operands ; i++)
                {
                    flag = (*(target_cell->formula))[i].type_flag;
                    if (flag == 0)
                    {
                        values[i] = (*(target_cell->formula))[i].operand_value.constant;
                    }
                    else
                    {
                        values[i] = (*(target_cell->formula))[i].operand_value.cell_operand->value;
                        err[i] = (*(target_cell->formula))[i].operand_value.cell_operand->is_error;
                    }
                }
                int temp = 0;
                bool exit_case = false;
                for(int i = 0; i < target_cell->count_operands; i++)
                {
                    temp = temp + values[i];
                    if(err[i]==true)
                    {
                        target_cell->is_error=true;
                        status = 2;
                        exit_case = true;
                        break;
                    }
                }
                if(exit_case)
                {
                    break;
                }
                target_cell->value = temp;
                status = 0;
                target_cell->is_error = false;
                break;
                }
        case 11:
                {
                int values[target_cell->count_operands];
                int err[target_cell->count_operands];
                int flag;
                for(int i = 0; i < target_cell->count_operands ; i++)
                {
                    flag = (*(target_cell->formula))[i].type_flag;
                    if (flag == 0)
                    {
                        values[i] = (*(target_cell->formula))[i].operand_value.constant;
                    }
                    else
                    {
                        values[i] = (*(target_cell->formula))[i].operand_value.cell_operand->value;
                        err[i] = (*(target_cell->formula))[i].operand_value.cell_operand->is_error;
                    }
                }
                // Calculate mean
                // double sum = 0.0;
                // bool exit_case = false;
                // for (int i = 0; i < target_cell->count_operands; i++)
                // {
                //     sum += values[i];
                //     if(err[i]==true)
                //     {
                //         target_cell->is_error = true;
                //         status = 2;
                //         exit_case = true;
                //         break;
                //     }
                // }
                // if(exit_case)
                // {
                //     break;
                // }
                // double mean = sum / target_cell->count_operands;
                
                // // Calculate squared differences and their sum
                // double sq_sum = 0.0;
                // for (int i = 0; i < target_cell->count_operands; i++)
                // {
                //     double diff = values[i] - mean;
                //     sq_sum += diff * diff;
                // }
                
                // // Calculate standard deviation (using population std deviation formula)
                // double std_dev = sqrt(sq_sum / target_cell->count_operands);
                
                // // Optionally round or convert to int if needed.
                bool exit_case = false;
                for(int i = 0; i < target_cell->count_operands; i++)
                {
                    if(err[i] == true)
                    {
                        target_cell->is_error = true;
                        status = 2;
                        exit_case = true;
                        break;
                    }
                }
                if(exit_case)
                {
                    break;
                }
                target_cell->value = (int)stdev(values, target_cell->count_operands);
                status = 0;
                target_cell->is_error = false;
                break;
                }
        case 12:
                {int seconds;

                if ((*(target_cell->formula))[0].type_flag == 0) {
                    seconds = (*(target_cell->formula))[0].operand_value.constant;
                } else {
                    seconds = (*(target_cell->formula))[0].operand_value.cell_operand->value;
                }
                fflush(stdout);
                // printf("---INSIDE SLEEP----\n");
                // printf("%d\n INT PRINTED",seconds);
                target_cell->value = handle_sleep(sheet,rt,ct,seconds);
                status = 0;
                // sleep(seconds);
                // printf("---OUTSIDE SLEEP----\n");
                target_cell->is_error = false;
                break;}
    }
}


//When a cell is assigned a formula 
//We need to update its attributes 
//Remove the old formula and assign the new formula
//Update number of operands 
//Remove its old precdents and delete teh cell from the dependent list of its precedents
//For all the cell in the cells in the formula add the current cell to their dependents list 

void assign_cell(Sheet* sheet, int r, int c,int operation_id, operand (*formula)[],int count_operands)
{   
    Cell* target_cell = sheet->all_cells[r][c];
    Cell** temp_precedents = NULL;
    int count = target_cell->count_precedents;
    // if (count > 0)
    // {
    //     temp_precedents  = (Cell**)malloc(sizeof(Cell*) * count);
    //     if (temp_precedents != NULL)
    //     {
    //         memcpy(temp_precedents, target_cell->precedents, sizeof(Cell*) * (count));
    //     }
    // }
    if (count > 0)
    {
        temp_precedents = (Cell**)malloc(sizeof(Cell*) * count);
        if (temp_precedents != NULL)
        {
            // Copy precedents from hash table to array
            HashIterator* iterator = hash_iterator_create(target_cell->precedents);
            Cell* precedent;
            int i = 0;
            while ((precedent = hash_iterator_next(iterator)) != NULL && i < count) {
                temp_precedents[i++] = precedent;
            }
            hash_iterator_free(iterator);
        }
    }
    // for(int i = 0; i < target_cell->count_precedents; i++)
    // {
    //     Cell* curr = (target_cell->precedents)[i];
    //     delete_depedency(sheet,curr->r,curr->c,r,c);
    // }
    HashIterator* iterator = hash_iterator_create(target_cell->precedents);
    Cell* precedent;
    while ((precedent = hash_iterator_next(iterator)) != NULL) {
        delete_depedency(sheet, precedent->r, precedent->c, r, c);
    }
    hash_iterator_free(iterator);

    clear_precedents(sheet, r, c);

    operand (*temp)[] = target_cell->formula;
    target_cell->formula = formula;
    int temp_id = target_cell->operation_id;
    target_cell->operation_id = operation_id;
    int temp_count = target_cell->count_operands;
    target_cell->count_operands = count_operands;
    for(int i = 0; i < count_operands; i++)
    {
        if((*formula)[i].type_flag == 1)
        {
            Cell* precedent_cell = (*formula)[i].operand_value.cell_operand;
            add_dependency(sheet,precedent_cell->r,precedent_cell->c,r,c);            
        }
    }
    if (has_cycle(sheet, sheet->all_cells[r][c]) == false)
    {
        if(zero_div_err(sheet, r, c) == false)
            {calculate_cell_value(sheet, r, c);
            recalculate_dependents(sheet, r, c);
            status = 0;
        }
        else{
            target_cell->is_error = true;
            status = 2;
            recalculate_dependents(sheet, r, c);
        }
    }
    else
    {
        status = 3;
        target_cell->formula = temp;
        target_cell->operation_id = temp_id;
        target_cell->count_operands = temp_count;
        for(int i = 0; i < count; i++)
        {
            Cell* curr = (temp_precedents)[i];
            add_dependency(sheet,curr->r,curr->c,r,c);
        }

        for(int i = 0; i < count_operands; i++)
        {
            if((*formula)[i].type_flag == 1)
            {
                Cell* precedent_cell = (*formula)[i].operand_value.cell_operand;
                delete_depedency(sheet,precedent_cell->r,precedent_cell->c,r,c);            
            }
        }

    }
    
    if(temp_precedents != NULL)
    {
        free(temp_precedents);
    }
    // print_formula(sheet, r, c);
    // printf("OPERATION ID : %d\n\n",sheet->all_cells[r][c]->operation_id);
}

int min(int a, int b)
{
    if(a <= b)
    {
        return a;
    }
    else
    {
        return b;
    }
}

int max(int a, int b)
{
    if(a >=  b)
    {
        return a;
    }
    else{
        return b;
    }
}

void print_formula(Sheet* sheet, int r, int c){
    Cell* curr_cell = sheet->all_cells[r][c];
    printf("--------FORMULA CONTENTS--------\n");
    for(int i = 0; i < curr_cell->count_operands; i++)
    {
        if((*(curr_cell->formula))[i].type_flag == 0)
        {
            printf("%d\n",((*(curr_cell->formula))[i].operand_value.constant));
        }
        else
        {
            printf("row = %d,",(((*(curr_cell->formula))[i].operand_value.cell_operand->r)));
            printf("column = %d",(((*(curr_cell->formula))[i].operand_value.cell_operand->c)));
            printf("\n");
        }
    }
    printf("--------FORMULA CONTENTS END --------\n");

}

bool precedent_has_error(Sheet* sheet, int r, int c){
    Cell* target_cell = sheet->all_cells[r][c];
    // for(int i = 0; i < target_cell->count_precedents; i++)
    // {
    //     Cell* curr_cell = target_cell->precedents[i];
    //     if (curr_cell->is_error == true)
    //     {
    //         status = 2;
    //         return true;
    //     }
    // }
    HashIterator* iterator = hash_iterator_create(target_cell->precedents);
    Cell* precedent;
    
    while ((precedent = hash_iterator_next(iterator)) != NULL) {
        if (precedent->is_error == true) {
            hash_iterator_free(iterator);
            status = 2;
            return true;
        }
    }
    
    hash_iterator_free(iterator);
    return false;
}

bool zero_div_err(Sheet* sheet, int r, int c)
{
    Cell* target_cell = sheet->all_cells[r][c];
    if(target_cell->operation_id == 6 && target_cell->count_precedents==2)
    {
        int flag = (*(target_cell->formula))[1].type_flag;
        int val2;
        if(flag == 0)
        {
            val2 = (*(target_cell->formula))[1].operand_value.constant;
        }
        else
        {
            val2 = (*(target_cell->formula))[1].operand_value.cell_operand->value;
        }


        if(val2 == 0)
        {
            status = 2;
            return true;
        }
        else{
            return false;
        }
    }
    return false;
}

// int handle_sleep(int seconds)
// {
//     // printf("Sleeping for %d seconds...\n", seconds);
//     fflush(stdout);  // Add this line
//     sleep(seconds);
//     // printf("Sleep completed.\n");
//     fflush(stdout);  // Add this line
//     return seconds;
// }

int stdev(int* arr, int n) {
    if (n <= 1) return 0;  // Avoid division by zero

    int sum = 0, mean;
    double variance = 0.0;

    // Calculate mean
    for (int i = 0; i < n; i++) {
        sum += arr[i];
    }
    mean = sum / n;

    // Calculate variance
    for (int i = 0; i < n; i++) {
        variance += (arr[i] - mean) * (arr[i] - mean);
    }
    variance /= n;

    // Return integer standard deviation (rounded)
    return (int)round(sqrt(variance));
}

void* sleep_thread_function(void* arg) {
    SleepThreadData* data = (SleepThreadData*)arg;
    
    // Sleep for the specified duration
    sleep(data->seconds);
    
    pthread_mutex_lock(&data->mutex);
    
    // Update the cell value after sleeping
    Cell* cell = data->sheet->all_cells[data->row][data->col];
    cell->value = data->seconds;
    data->completed = true;
    
    // Signal that sleep is completed
    pthread_cond_signal(&data->condition);
    pthread_mutex_unlock(&data->mutex);
    
    return NULL;
}

int handle_sleep(Sheet* sheet, int row, int col, int seconds) {
    // Create thread data
    SleepThreadData* thread_data = malloc(sizeof(SleepThreadData));
    thread_data->sheet = sheet;
    thread_data->row = row;
    thread_data->col = col;
    thread_data->seconds = seconds;
    thread_data->completed = false;
    
    // Initialize synchronization primitives
    pthread_mutex_init(&thread_data->mutex, NULL);
    pthread_cond_init(&thread_data->condition, NULL);
    
    // Create the thread
    pthread_t thread;
    pthread_create(&thread, NULL, sleep_thread_function, thread_data);
    
    // Detach the thread so we don't need to wait for it
    pthread_detach(thread);
    
    // Return immediately, cell value will be updated when thread completes
    return seconds;
}