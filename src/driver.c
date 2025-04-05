#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <limits.h>
#include <time.h>
#include <stdbool.h>
#include <unistd.h>
#include <math.h>

// Include headers from driver.c (assuming these are necessary)
#include "dependency_graph_final.h"  // Assuming this is a local header
#include "cell.h"
#include "input.h"             // Assuming this is a local header
// --- Definitions and Structures from display.c ---
#define MAX_ROWS 999
#define MAX_COLS 18278
#define VIEWPORT_SIZE 10

int current_row = 0, current_col = 0; // Current focused cell
int display_cell = 0, display_row = 0, display_column = 0; // Display mode counters

// --- Function Declarations (for functions from display.c) ---
void get_col_label(int col, char *label);
void display_sheet(Sheet* sheet, int rows, int cols); // Modified to take Sheet*
// void display_single_cell(Sheet* sheet, int row, int col); // Modified



int main(int argc, char* argv[]) {
    if (argc != 3) {
        printf("Out fo bounds rows or columns");
        exit(1);
    }

    int rows = atoi(argv[1]);
    int columns = atoi(argv[2]);

    if (!(rows >= 1 && rows <= MAX_ROWS && columns >= 1 && columns <= MAX_COLS)) {
        printf("Out fo bounds rows or columns");
        exit(1);
    }

    Sheet* sheet;
    sheet = initialise(rows, columns);

    char input[256];
    char status_str[256];
    double execution_time = 0.0;
    bool print_flag = true;
    display_sheet(sheet, rows, columns);
    

    while (1) {
        if(status==0){
            strcpy(status_str,"ok");
        }
        else if(status==1){
            strcpy(status_str,"Invalid Input");
            status=0;
        }
        else if(status==2){
            strcpy(status_str,"ok");
            status=0;
        }
        else if(status==3){
            strcpy(status_str,"cyclic dependence");
            status=0;
        }
        printf("[%.1f] (%s) > ", execution_time, status_str);
        fgets(input, sizeof(input), stdin);
        input[strcspn(input, "\n")] = '\0';

        clock_t start = clock();

        if (strcmp(input, "q") == 0) {
            break;
        } else if (strcmp(input, "disable_output") == 0) {
            print_flag = false;
        } else if (strcmp(input, "enable_output") == 0) {
            print_flag = true;
        } else if (strcmp(input, "w") == 0) {

            if(current_row-10 < 0){
                current_row=0;
            }
            else{
                current_row=current_row-10;
            }
            
        } else if (strcmp(input, "s") == 0) {
            if(current_row+10 >rows){
                current_row=current_row;
            }
            else if(current_row+20>rows){
                current_row=rows-10;
            }
            else{
                current_row=current_row+10;
            }


        } else if (strcmp(input, "a") == 0) {
            if(current_col-10 < 0){
                current_col=0;
            }
            else{
                current_col=current_col-10;
            }

        } else if (strcmp(input, "d") == 0) {
            if(current_col+10 >columns){
                current_col=current_col;
            }
            else if(current_col+20>columns){
                current_col=columns-10;
            }
            else{
                current_col=current_col+10;
            }

        } else if (strncmp(input, "scroll_to ", 10) == 0) {
            int new_row, new_col;
            parseCellName(input + 10, &new_row, &new_col);
            if (new_row >= 0 && new_row < rows && new_col >= 0 && new_col < columns) {
                current_row = new_row;
                current_col = new_col;
                display_row = 0;
                display_column = 0;
            }
            else{
                printf("Out fo bounds rows or columns");
            }
        } else {
          parseInput(input, sheet, rows, columns);
        }
        //parseInput(input, sheet, rows, columns, print_flag); // Remove the print flag
        //assign_cell(sheet, editrow, editcolumn, operationID, formula, count_operands);
        if((strcmp(input, "enable_output") != 0) || (strcmp(input, "disable_output") != 0) || (strncmp(input, "scroll", 7) != 0))
        {
            if(status!=1){
            assign_cell(sheet, editrow, editcolumn, operationID, formula, count_operands);
            }
        }

        clock_t end = clock();
        execution_time = (double)(end - start) / CLOCKS_PER_SEC;
        

        if (print_flag == false) {
            continue;
        }
        
        display_sheet(sheet,rows,columns); // Update the display after each command
    }

    // Free allocated memory (VERY IMPORTANT!) - Add the deallocation function from dependency_graph.c
    //free_sheet(sheet);

    return 0;
}



// --- Function from display.c ---
void get_col_label(int col, char *label) {
    int index = 0;
    col++; // 1-based index
    while (col > 0) {
        col--;
        label[index++] = 'A' + (col % 26);
        col /= 26;
    }
    label[index] = '\0';
    // Reverse the label
    for (int i = 0; i < index / 2; i++) {
        char temp = label[i];
        label[i] = label[index - i - 1];
        label[index - i - 1] = temp;
    }
}

// --- Modified display functions to work with the Sheet struct ---
void display_sheet(Sheet* sheet, int rows, int cols) {
    
    printf("    ");
    for (int c = current_col; c < current_col + VIEWPORT_SIZE && c < cols; c++) {
        char label[4] = "";
        get_col_label(c, label);
        printf("%9s", label);
    }
    printf("\n");

    for (int r = current_row; r < current_row + VIEWPORT_SIZE && r < rows; r++) {
        printf("%3d", r + 1);
        for (int c = current_col; c < current_col + VIEWPORT_SIZE && c < cols; c++) {
            if(sheet->all_cells[r][c]->is_error==true){
                char *error="ERR";
                printf("%9s",error);
            }
            else{
            printf("%9d", sheet->all_cells[r][c]->value); // Access value through the Sheet struct
            }
        }
        printf("\n");
    }
}

// void display_single_cell(Sheet* sheet, int row, int col) {
    
//     printf("   ");
//     char label[4] = "";
//     get_col_label(col, label);
//     printf(" %9s\n", label);
//     if(sheet->all_cells[row][col]->is_error==true){
//         char *error="ERR";
//         printf("%3d %9s \n", row + 1,error);
//     }
//     else{
//     printf("%3d %9d\n", row + 1, sheet->all_cells[row][col]->value); // Access value
//     }
// }




