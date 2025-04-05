
CC = gcc
CFLAGS = -Wall -Wextra -O3 -flto
LDFLAGS = -lm -flto
DEPS = dependency_graph_final.h input.h cell.h hash_table.h

BUILD_DIR = target
RELEASE_DIR = $(BUILD_DIR)/release

all: $(RELEASE_DIR)/spreadsheet test_runner

$(RELEASE_DIR)/spreadsheet: driver.c dependency_graph_final.c input.c hash_table.c $(DEPS)
	@mkdir -p $(RELEASE_DIR)
	$(CC) -o $(RELEASE_DIR)/spreadsheet driver.c dependency_graph_final.c input.c hash_table.c $(CFLAGS) $(LDFLAGS)

test_runner: test_runner.c
	$(CC) -o test_runner test_runner.c $(CFLAGS) $(LDFLAGS)

test: $(RELEASE_DIR)/spreadsheet test_runner
	./test_runner test_cases.txt expected_output.txt

report: report.tex
	pdflatex report.tex
	pdflatex report.tex  # Run twice to fix references

clean:
	rm -rf $(BUILD_DIR) test_runner output.txt report.aux report.log report.out report.pdf sheet