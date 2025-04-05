#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>
#include <fcntl.h>
#include <stdbool.h>

#define MAX_LINE_LENGTH 256
#define MAX_TEST_CASES 15
#define MAX_COMMANDS_PER_TEST 200

// Structure to store a test case
typedef struct {
    int rows;
    int cols;
    char commands[MAX_COMMANDS_PER_TEST][MAX_LINE_LENGTH];
    int num_commands;
} TestCase;

// Function declarations
void parse_test_cases(const char* filename, TestCase test_cases[], int* num_test_cases);
void run_test_case(TestCase* test_case, int test_num, FILE* output_file);
bool compare_outputs(const char* actual_file, const char* expected_file, int test_num);
void clean_string(char* str);

int main(int argc, char* argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <test_cases_file> <expected_output_file>\n", argv[0]);
        return 1;
    }

    const char* test_file = argv[1];
    const char* expected_file = argv[2];
    const char* output_file = "output.txt";
    
    TestCase test_cases[MAX_TEST_CASES];
    int num_test_cases = 0;
    
    // Parse test cases from file
    parse_test_cases(test_file, test_cases, &num_test_cases);
    
    // Create/truncate output file
    FILE* output_fp = fopen(output_file, "w");
    if (!output_fp) {
        perror("Error opening output file");
        return 1;
    }
    
    // Run each test case
    printf("Running %d test cases...\n", num_test_cases);
    for (int i = 0; i < num_test_cases; i++) {
        run_test_case(&test_cases[i], i + 1, output_fp);
    }
    
    fclose(output_fp);
    
    // Compare outputs with expected output
    bool all_passed = true;
    for (int i = 0; i < num_test_cases; i++) {
        bool passed = compare_outputs(output_file, expected_file, i + 1);
        if (!passed) {
            all_passed = false;
        }
    }
    
    if (all_passed) {
        printf("\nAll test cases passed!\n");
        return 0;
    } else {
        printf("\nSome test cases failed. Check details above.\n");
        return 1;
    }
}

// Parse test cases from the input file
void parse_test_cases(const char* filename, TestCase test_cases[], int* num_test_cases) {
    FILE* fp = fopen(filename, "r");
    if (!fp) {
        perror("Error opening test cases file");
        exit(1);
    }
    
    char line[MAX_LINE_LENGTH];
    int test_idx = -1;
    int blank_line_count = 0;
    
    while (fgets(line, sizeof(line), fp)) {
        // Remove newline character
        clean_string(line);
        
        // Skip empty lines, but count consecutive empty lines
        if (strlen(line) == 0) {
            blank_line_count++;
            continue;
        }
        
        // Reset blank line count on non-empty line
      
        
        // If first line of test or after 2 blank lines, start new test case
        if (test_idx == -1 || (blank_line_count == 2 && test_cases[test_idx].num_commands > 0)) {
            blank_line_count=0;
            test_idx++;
            *num_test_cases = test_idx + 1;
            
            if (test_idx >= MAX_TEST_CASES) {
                fprintf(stderr, "Too many test cases. Maximum allowed: %d\n", MAX_TEST_CASES);
                break;
            }
            
            // Parse first line for rows and columns
            if (sscanf(line, "%d %d", &test_cases[test_idx].rows, &test_cases[test_idx].cols) != 2) {
                fprintf(stderr, "Error parsing dimensions for test case %d\n", test_idx + 1);
                exit(1);
            }
            
            test_cases[test_idx].num_commands = 0;
            continue;
        }
        
        // Add command to current test case
        if (test_cases[test_idx].num_commands < MAX_COMMANDS_PER_TEST) {
            strcpy(test_cases[test_idx].commands[test_cases[test_idx].num_commands], line);
            test_cases[test_idx].num_commands++;
        } else {
            fprintf(stderr, "Too many commands for test case %d. Maximum allowed: %d\n", 
                    test_idx + 1, MAX_COMMANDS_PER_TEST);
        }
    }
    
    fclose(fp);
    printf("Successfully parsed %d test cases.\n", *num_test_cases);
}

// Run a single test case
void run_test_case(TestCase* test_case, int test_num, FILE* output_file) {
    printf("Running test case %d (Sheet: %d x %d, Commands: %d)...\n", 
           test_num, test_case->rows, test_case->cols, test_case->num_commands);
    
    // Create pipes for communication with the child process
    int input_pipe[2];  // Parent writes to this pipe
    int output_pipe[2]; // Parent reads from this pipe
    
    if (pipe(input_pipe) < 0 || pipe(output_pipe) < 0) {
        perror("Pipe creation failed");
        exit(1);
    }
    
    // Make output pipe non-blocking
    fcntl(output_pipe[0], F_SETFL, O_NONBLOCK);
    
    // Fork a child process to run the sheet program
    pid_t pid = fork();
    
    if (pid < 0) {
        perror("Fork failed");
        exit(1);
    }
    
    if (pid == 0) {
        // Child process
        
        // Redirect stdin to read from input_pipe
        close(input_pipe[1]); // Close write end
        dup2(input_pipe[0], STDIN_FILENO);
        close(input_pipe[0]);
        
        // Redirect stdout to write to output_pipe
        close(output_pipe[0]); // Close read end
        dup2(output_pipe[1], STDOUT_FILENO);
        close(output_pipe[1]);
        
        // Convert dimensions to strings
        char rows_str[16], cols_str[16];
        sprintf(rows_str, "%d", test_case->rows);
        sprintf(cols_str, "%d", test_case->cols);
        
        // Execute the sheet program
        execl("./sheet", "./sheet", rows_str, cols_str, NULL);
        
        // If execl fails
        perror("Exec failed");
        exit(1);
    } else {
        // Parent process
        
        // Close unused pipe ends
        close(input_pipe[0]);
        close(output_pipe[1]);
        
        // Write commands to the input pipe
        for (int i = 0; i < test_case->num_commands; i++) {
            // Add newline to command if not already present
            char cmd[MAX_LINE_LENGTH + 2];
            strcpy(cmd, test_case->commands[i]);
            if (cmd[strlen(cmd) - 1] != '\n') {
                strcat(cmd, "\n");
            }
            
            write(input_pipe[1], cmd, strlen(cmd));
            
            // Add a small delay to allow processing
            usleep(100000); // 100ms
        }
        
        // Send quit command
        write(input_pipe[1], "q\n", 2);
        close(input_pipe[1]);
        
        // Read output from the child process
        char buffer[4096];
        int bytes_read;
        
        // Wait a bit to ensure output is ready
        usleep(500000); // 500ms
        
        // Mark the beginning of this test case in the output file
        fprintf(output_file, "=== TEST CASE %d ===\n", test_num);
        
        // Read and write all available output
        while ((bytes_read = read(output_pipe[0], buffer, sizeof(buffer) - 1)) > 0) {
            buffer[bytes_read] = '\0';
            fprintf(output_file, "%s", buffer);
            usleep(100000); // Small delay
        }
        
        // Mark the end of this test case
        fprintf(output_file, "\n=== END TEST CASE %d ===\n\n", test_num);
        
        // Close remaining pipe
        close(output_pipe[0]);
        
        // Wait for child process to terminate
        int status;
        waitpid(pid, &status, 0);
        
        if (WIFEXITED(status)) {
            printf("Test case %d completed with exit code: %d\n", test_num, WEXITSTATUS(status));
        } else {
            printf("Test case %d terminated abnormally\n", test_num);
        }
    }
}

// Compare actual output with expected output for a specific test case
bool compare_outputs(const char* actual_file, const char* expected_file, int test_num) {
    FILE* actual_fp = fopen(actual_file, "r");
    FILE* expected_fp = fopen(expected_file, "r");
    
    if (!actual_fp || !expected_fp) {
        if (!actual_fp) perror("Error opening actual output file");
        if (!expected_fp) perror("Error opening expected output file");
        if (actual_fp) fclose(actual_fp);
        if (expected_fp) fclose(expected_fp);
        return false;
    }
    
    char actual_line[MAX_LINE_LENGTH];
    char expected_line[MAX_LINE_LENGTH];
    int line_num = 0;
    bool in_target_test = false;
    bool test_start_found = false;
    bool test_passed = true;
    
    // Find the beginning of the target test case in both files
    char test_marker[50];
    sprintf(test_marker, "=== TEST CASE %d ===", test_num);
    
    // Find the start marker in actual output
    while (fgets(actual_line, sizeof(actual_line), actual_fp)) {
        clean_string(actual_line);
        if (strcmp(actual_line, test_marker) == 0) {
            in_target_test = true;
            break;
        }
    }
    
    // Find the start marker in expected output
    while (fgets(expected_line, sizeof(expected_line), expected_fp)) {
        clean_string(expected_line);
        if (strcmp(expected_line, test_marker) == 0) {
            test_start_found = true;
            break;
        }
    }
    
    if (!in_target_test || !test_start_found) {
        printf("Test case %d: Could not find test case markers in output files\n", test_num);
        fclose(actual_fp);
        fclose(expected_fp);
        return false;
    }
    
    // Compare the lines until the end marker
    sprintf(test_marker, "=== END TEST CASE %d ===", test_num);
    
    while (true) {
        bool actual_eof = fgets(actual_line, sizeof(actual_line), actual_fp) == NULL;
        bool expected_eof = fgets(expected_line, sizeof(expected_line), expected_fp) == NULL;
        
        // Clean up the lines
        if (!actual_eof) clean_string(actual_line);
        if (!expected_eof) clean_string(expected_line);
        
        // Check for end marker
        if (!actual_eof && strcmp(actual_line, test_marker) == 0) {
            break;
        }
        
        if (!expected_eof && strcmp(expected_line, test_marker) == 0) {
            break;
        }
        
        // If one file ends before the other
        if (actual_eof != expected_eof) {
            printf("Test case %d: Files have different length\n", test_num);
            test_passed = false;
            break;
        }
        
        if (actual_eof && expected_eof) {
            break;
        }
        
        line_num++;
        
        // Compare the lines
        if (strcmp(actual_line, expected_line) != 0) {
            printf("Test case %d: Mismatch at line %d\n", test_num, line_num);
            printf("  Expected: '%s'\n", expected_line);
            printf("  Actual  : '%s'\n", actual_line);
            test_passed = false;
        }
    }
    
    fclose(actual_fp);
    fclose(expected_fp);
    
    if (test_passed) {
        printf("Test case %d: PASSED\n", test_num);
    } else {
        printf("Test case %d: FAILED\n", test_num);
    }
    
    return test_passed;
}

// Remove newline characters and trailing whitespace
void clean_string(char* str) {
    if (!str) return;
    
    size_t len = strlen(str);
    if (len == 0) return;
    
    // Remove trailing newline and carriage return
    if (str[len - 1] == '\n' || str[len - 1] == '\r') {
        str[len - 1] = '\0';
        len--;
    }
    
    if (len > 0 && str[len - 1] == '\r') {
        str[len - 1] = '\0';
    }
}