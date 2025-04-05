#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <limits.h>
#include <math.h>
#include <stdbool.h>
#include "dependency_graph_final.h"



int operationID;
int editrow;
int editcolumn;
int count_operands;
operand (*formula)[];


// Function Prototypes

void parseCellName(const char* cellName, int* row, int* col);
void parseInput(const char* input,Sheet* spreadsheet , int rows, int cols);
int isArithmeticExpression(const char* expression);
int isFunction(const char* expression);
int AssignValue(char *op);
bool contains_alphabet(const char *str);
int string_to_int(const char *num_str);
int count_occurrences(char ch, const char *str) ;


// Convert cell name (e.g., A1) to row and column indices
void parseCellName(const char* cellName, int* row, int* col) {
    int i = 0;
    *col = 0;

    // Extract column part (letters)
    while (true) {
        if (cellName[i]-'0' >=0 && cellName[i]-'0' <=9){
            break;
        }
        else if (cellName[i]-'A'>=26 || cellName[i]-'A'<0  ){
            status = 1;
            // printf("invalidInput");
            break;
        }
        *col = *col * 26 + (toupper(cellName[i]) - 'A' + 1);
        i++;
    }
    *col-=1;

    // Extract row part (numbers)
    if (!contains_alphabet(&cellName[i]) && !isArithmeticExpression(&cellName[i])){
    *row = string_to_int(&cellName[i]);
    *row-=1;}
    else{
        status = 1;
        // printf("invalidInput");
    }
}

// Parse input command
void parseInput(const char* input,Sheet* spreadsheet , int rows, int cols){
    char cellName[16];
    char expression[128];
    

    if (sscanf(input, "%[^=]=%s", cellName, expression) == 2) {
       
      
        parseCellName(cellName, &editrow, &editcolumn);

        if (editrow < 0 || editrow > rows || editcolumn < 0 || editcolumn > cols) { 
            status = 1;                                                                          
            // printf("Error: Invalid cell reference.\n");
            return;
        }

        if (isdigit(expression[0]) || expression[0] == '-' || expression[0] == '+'||( isalpha(expression[0]) && !strchr(expression,'(')) ) {
            if((!contains_alphabet(expression)) && !isArithmeticExpression(expression+1)){
                count_operands=1;
                formula=(operand (*)[])malloc(sizeof(operand));
                (*formula)[0].type_flag = 0; // Constant
                (*formula)[0].operand_value.constant = string_to_int(expression);
                operationID = 1; // Cell assignment with a constant
            }  
            else if (contains_alphabet(expression) && !isArithmeticExpression(expression+1))  {
                int row1,col1;
                parseCellName(expression, &row1, &col1);
                if (row1 < 0 || row1 > rows || col1 < 0 || col1 > cols) { 
                    // status  = 1;                                                                          
                    // printf("Error: Invalid cell reference.\n");
                    return;
                }
                // printf("---CELL NAMES\n");
                // printf("row = %d, column = %d", row1,col1);
                // printf("---CELL NAMES END\n");

                if (row1 < 0 || row1 > rows || col1 < 0 || col1 > cols) {
                    status = 1;
                    // printf("Error: Invalid cell reference.\n");
                    return;
                }
                count_operands=1;
                formula=(operand (*)[])malloc(sizeof(operand));
                (*formula)[0].type_flag = 1; // Constant
                (*formula)[0].operand_value.cell_operand = spreadsheet->all_cells[row1][col1];
                operationID = 2; // Cell assignment with a constant

            }  
            else if ( contains_alphabet(expression) && isArithmeticExpression(expression+1)) {
                count_operands=2;
                formula=(operand (*)[])malloc(sizeof(operand)*2);
                char operand1[16], operand2[16], op;
                if (!(expression[0]=='+'|| expression[0]=='-')){

                    if (sscanf(expression, "%[^+-*/]%c%s", operand1, &op, operand2) == 3) {
                        int val1, val2;
                        if (isalpha(operand1[0])) {
                            int r1, c1;
                            parseCellName(operand1, &r1, &c1);
                            if (r1 < 0 || r1 > rows || c1 < 0 || c1 > cols) {       
                                status = 1;                                                                    
                                // printf("Error: Invalid cell reference.\n");
                                return;
                            }
                            (*formula)[0].type_flag = 1; // Constant
                            (*formula)[0].operand_value.cell_operand = spreadsheet->all_cells[r1][c1] ;
                            
                            operationID = AssignValue(&op); // Cell assignment with a constant  
                            
                        } else {
                            (*formula)[0].type_flag = 0; // Constant
                            (*formula)[0].operand_value.constant = string_to_int(operand1) ;
                            operationID = AssignValue(&op); // Cell assignment with a constant
                        }

                        if (isalpha(operand2[0])) {
                            int r1, c1;
                            parseCellName(operand2, &r1, &c1);
                            if (r1 < 0 || r1 > rows || c1 < 0 || c1 > cols) {    
                                status = 1;                                                                       
                                // printf("Error: Invalid cell reference.\n");
                                return;
                            }
                            (*formula)[1].type_flag = 1; // Constant
                            (*formula)[1].operand_value.cell_operand = spreadsheet->all_cells[r1][c1] ;
                            operationID = AssignValue(&op); // Cell assignment with a constant
                            
                        } else {
                            (*formula)[1].type_flag = 0; // Constant
                            (*formula)[1].operand_value.constant = string_to_int(operand2) ;
                            operationID = AssignValue(&op); // Cell assignment with a constant
                        }
                      
                    }
                    else{
                        status = 1; 
                        // printf("invalid input");
                    }
            }
            else if((expression[0]=='+'|| expression[0]=='-')){
                if(isalpha(expression[1]) || expression[1] == '-' || expression[1]=='+'){
                    status = 1; 
                    // printf("invalid_input");
                }
                if (sscanf(expression+1, "%[^+-*/]%c%s", operand1, &op, operand2) == 3) {
                    int val1, val2;
                    if (isalpha(operand1[0])) {
                        int r1, c1;
                        parseCellName(operand1, &r1, &c1);
                        if (r1 < 0 || r1 > rows || c1 < 0 || c1 > cols) {  
                            status = 1;                                                                          
                            // printf("Error: Invalid cell reference.\n");
                            return;
                        }
                        (*formula)[0].type_flag = 1; // Constant
                        (*formula)[0].operand_value.cell_operand = spreadsheet->all_cells[r1][c1] ;
                        operationID = AssignValue(&op); // Cell assignment with a constant  // remember here emre ko operation id assignment karna hai with +,-,/,*
                        
                    } else {
                        int value=string_to_int(operand1);
                        if(expression[0]=='-'){
                            value=value*(-1);
                        }
                        (*formula)[0].type_flag = 0; // Constant
                        (*formula)[0].operand_value.constant = value ;
                        operationID = AssignValue(&op); // Cell assignment with a constant
                    }

                    if (isalpha(operand2[0])) {
                        int r1, c1;
                        parseCellName(operand2, &r1, &c1);
                        if (r1 < 0 || r1 > rows || c1 < 0 || c1 > cols) {     
                            status = 1;                                                                      
                            // printf("Error: Invalid cell reference.\n");
                            return;
                        }
                        (*formula)[1].type_flag = 1; // Constant
                        (*formula)[1].operand_value.cell_operand = spreadsheet->all_cells[r1][c1];
                        operationID = AssignValue(&op); // Cell assignment with a constant
                        
                    } else {
                        (*formula)[1].type_flag = 0; // Constant
                        (*formula)[1].operand_value.constant = string_to_int(operand2) ;
                        operationID = AssignValue(&op); // Cell assignment with a constant
                    }
                  
                }
                else{
                    status = 1; 
                    // printf("invalid input");
                }
           } 
        } 
        else if ((!contains_alphabet(expression)) && isArithmeticExpression(expression + 1)){
            count_operands=2;
            formula=(operand (*)[])malloc(sizeof(operand)*2);
            char operand1[16], operand2[16], op;
            if (!(expression[0]=='+'|| expression[0]=='-')){

                if (sscanf(expression,"%[^+-*/]%c%s", operand1, &op, operand2) == 3) {
                    int val1, val2;
                    if (isalpha(operand1[0])) {
                        int r1, c1;
                        parseCellName(operand1, &r1, &c1);
                        if (r1 < 0 || r1 > rows || c1 < 0 || c1 > cols) {        
                            status = 1;                                                                    
                            // printf("Error: Invalid cell reference.\n");
                            return;
                        }
                        (*formula)[0].type_flag = 1; // Constant
                        (*formula)[0].operand_value.cell_operand = spreadsheet->all_cells[r1][c1] ;
                        
                        operationID = AssignValue(&op); // Cell assignment with a constant  
                        
                    } else {
                        (*formula)[0].type_flag = 0; // Constant
                        (*formula)[0].operand_value.constant = string_to_int(operand1) ;
                        operationID = AssignValue(&op); // Cell assignment with a constant
                    }

                    if (isalpha(operand2[0])) {
                        int r1, c1;
                        parseCellName(operand2, &r1, &c1);
                        if (r1 < 0 || r1 > rows || c1 < 0 || c1 > cols) {          
                            status = 1;                                                                  
                            // printf("Error: Invalid cell reference.\n");
                            return;
                        }
                        (*formula)[1].type_flag = 1; // Constant
                        (*formula)[1].operand_value.cell_operand = spreadsheet->all_cells[r1][c1] ;
                        operationID = AssignValue(&op); // Cell assignment with a constant
                        
                    } else {
                        (*formula)[1].type_flag = 0; // Constant
                        (*formula)[1].operand_value.constant = string_to_int(operand2) ;
                        operationID = AssignValue(&op); // Cell assignment with a constant
                    }
                    
                }
                else{
                    status = 1; 
                    // printf("invalid input");
                }
            }
            else if((expression[0]=='+'|| expression[0]=='-')){
                if(expression[1]=='+' || expression[1]=='-' ){
                    status = 1; 
                    // printf("invalid input");
                    return;
                }
                if (sscanf(expression+1, "%[^+-*/]%c%s", operand1, &op, operand2) == 3) {
                    int val1, val2;
                    if (isalpha(operand1[0])) {
                        int r1, c1;
                        parseCellName(operand1, &r1, &c1);
                        if (editrow < 0 || editrow > rows || editcolumn < 0 || editcolumn > cols) { 
                            status = 1;                                                                           
                            // printf("Error: Invalid cell reference.\n");
                            return;
                        }
                        (*formula)[0].type_flag = 1; // Constant
                        (*formula)[0].operand_value.cell_operand = spreadsheet->all_cells[r1][c1] ;
                        operationID = AssignValue(&op); // Cell assignment with a constant  // remember here emre ko operation id assignment karna hai with +,-,/,*
                        
                    } else {
                        int value=string_to_int(operand1);
                        if(expression[0]=='-'){
                            value=value*(-1);
                        }
                        (*formula)[0].type_flag = 0; // Constant
                        (*formula)[0].operand_value.constant = value ;
                        operationID = AssignValue(&op); // Cell assignment with a constant
                    }

                    if (isalpha(operand2[0])) {
                        int r1, c1;
                        parseCellName(operand2, &r1, &c1);
                        if (r1 < 0 || r1 > rows || c1 < 0 || c1 > cols) {       
                            status = 1;                                                                     
                            // printf("Error: Invalid cell reference.\n");
                            return;
                        }
                        (*formula)[1].type_flag = 1; // Constant
                        (*formula)[1].operand_value.cell_operand = spreadsheet->all_cells[r1][c1];
                        operationID = AssignValue(&op); // Cell assignment with a constant
                        
                    } else {
                        (*formula)[1].type_flag = 0; // Constant
                        (*formula)[1].operand_value.constant = string_to_int(operand2) ;
                        operationID = AssignValue(&op); // Cell assignment with a constant
                    }
                  
                }
           } 
            }
            else{
                status = 1; 
                // printf("invalid input");
                return;
            }
        } else if (isFunction(expression)) {
            char *openParen = strchr(expression, '(');  // Find first '('
            char *closeParen = strrchr(expression, ')'); // Find last ')'
        
            // Check if '(' and ')' exist and in correct order
            if (!openParen || !closeParen || openParen > closeParen - 1) {
                status = 1; 
                // printf("invalid input");
            }

            int open=count_occurrences('(',expression);
            int close=count_occurrences(')',expression);
            if(open!=close){
                status=1;
            }
          
            
            char functionName[16], range[64];
            if (sscanf(expression, "%[^()](%[^)])", functionName, range) == 2) {
                if (strcmp(functionName, "MIN") == 0){
                    operationID=7;
                }
                else if (strcmp(functionName, "MAX") == 0){
                    operationID=8;
                }
                else if (strcmp(functionName, "AVG") == 0){
                    operationID=9;                
                }
                else if (strcmp(functionName, "SUM") == 0){
                    operationID=10;      
                }
                else if (strcmp(functionName, "STDEV") == 0){
                    operationID=11;  
                }
                else if (strcmp(functionName, "SLEEP") == 0){
                    operationID=12;  
                }
                if(strcmp(functionName, "SLEEP") == 0){
                    count_operands=1;
                    formula=(operand (*)[])malloc(sizeof(operand));
                    char extractedvalue[16];
                    if (contains_alphabet(range)) {
                        int r3, c3;
                        parseCellName(range, &r3, &c3);
                        if (r3 < 0 || r3 > rows || c3 < 0 || c3 > cols) {  
                            status = 1;                                                                          
                            // printf("Error: Invalid cell reference.\n");
                            return;
                        }
                        (*formula)[0].type_flag = 1; // Constant
                        (*formula)[0].operand_value.cell_operand = spreadsheet->all_cells[r3][c3] ;             
                    } else {
                        (*formula)[0].type_flag = 0; // Constant
                        (*formula)[0].operand_value.constant = string_to_int(range) ;
                       
                    }
                

                }
                else{
                   
                    int r3, c3, r4, c4;
                    char start[16], end[16];
                    if (sscanf(range, "%[^:]:%s", start, end) == 2) {
                        parseCellName(start, &r3, &c3);
                        if (r3 < 0 || r3 > rows || c3 < 0 || c3 > cols) {    
                            status = 1;                                                                        
                            // printf("Error: Invalid cell reference.\n");
                            return;
                        }
                        parseCellName(end, &r4, &c4);
                        if (r4 < 0 || r4 > rows || c4 < 0 || c4 > cols) {   
                            status = 1;                                                                         
                            // printf("Error: Invalid cell reference.\n");
                            return;
                        }
                        
                        if(r3>r4 || c3 > c4){ 
                            status = 1;
                            //  printf("Invalid range");
                            return ;}
                        count_operands=(r4-r3+1)*(c4-c3+1);
                        formula=(operand (*)[])malloc(sizeof(operand)*(r4-r3+1)*(c4-c3+1));
                       
                        int count=0;
                            for (int i = r3; i <= r4; i++) {
                                for (int j = c3; j <= c4; j++) {
                                    (*formula)[count].type_flag = 1; // Constant
                                    (*formula)[count].operand_value.cell_operand = spreadsheet->all_cells[i][j] ;
                                    count++;
                                  
                                } 
                            }
                        
                        }
                        else{
                            status = 1; 
                            // printf("invalid input");
                        }
                    }
                }
                else{
                    status = 1; 
                    // printf("invalid input");
                }
       
            } else {
                status = 1; 
                // printf("Error: Invalid function call.\n");
            }
        } 
        else{
            status=1;
        }
    

}

// Check if an expression is arithmetic
int isArithmeticExpression(const char* expression) {
    return strpbrk(expression, "+-*/") != NULL;
}

// Check if an expression is a function
int isFunction(const char* expression) {
    return strstr(expression, "MIN(") || strstr(expression, "MAX(") ||
           strstr(expression, "AVG(") || strstr(expression, "SUM(") || strstr(expression, "STDEV(") || strstr(expression, "SLEEP(");
}

// Perform arithmetic operation
int AssignValue(char *op){
 if (*op =='+'){return 3;}
 else if (*op =='-'){return 4;}
 else if (*op =='*'){return 5;}
 else if (*op =='/'){return 6;}
 else {
    // printf("Error");
    status=1;
 }

}


bool contains_alphabet(const char *str) {
    if (str == NULL) {
        return false;
    }

    while (*str != '\0') {
        if (isalpha(*str)) {
            return true;
        }
        str++;
    }

    return false;
}




int string_to_int(const char *num_str) {
    int num = 0, i = 0, sign = 1;

    // Check for negative sign
    if (num_str[0] == '-') {
        sign = -1;
        i = 1;  // Start conversion from the next character
    }

    if (num_str[0] == '+') {
        i = 1;  // Start conversion from the next character
    }

    // Convert string to integer
    while (num_str[i] != '\0') {
        if(num_str[i]-'0' < 0 || num_str[i] -'0' > 9){
            status = 1; 
            // printf("invalid_input");
            break;
        }
        num = num * 10 + (num_str[i] - '0');  
        i++;
    }

    return num * sign;  // Apply sign and return result
}

int count_occurrences(char ch, const char *str) {
    int count = 0;
    while (*str) {
        if (*str == ch) {
            count++;
        }
        str++;
    }
    return count;
}

