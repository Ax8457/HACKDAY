#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

void hide(const char *filename, int start_pos, const char *message) {
	FILE *file = fopen(filename, "r+b"); 
	if (!file) {
		perror("[x] Error opening file");
		return;
	}
	
	fseek(file, 0, SEEK_END);
	long file_size = ftell(file); 
	if (file_size <= start_pos) {
		printf("[x] Start position is beyond file size!\n");
		fclose(file);
		return;
	}
	
	fseek(file, start_pos, SEEK_SET); 
	
	size_t message_length = strlen(message);
	int total_bits = message_length * 8; 
	int *positions = malloc(total_bits * sizeof(int)); 
	if (!positions) {
		perror("[x] Memory allocation error");
		fclose(file);
		return;
	}
	
	srand(start_pos); 
	unsigned char byte;
	
	FILE *pos_file = fopen("pos.txt", "w");
	if (!pos_file) {
		perror("[x] Error opening pos.txt");
		free(positions);
		fclose(file);
		return;
	}
	
	for (int i = 0; i < total_bits; i++) {
		int bit = (message[i / 8] >> (7 - (i % 8))) & 1; 
		
		int pos;
		do {
			pos = start_pos + (rand() % (file_size - start_pos)); 
		} while (fseek(file, pos, SEEK_SET) != 0); 
		
		if (fread(&byte, 1, 1, file) != 1) {
			perror("[x] Error reading file");
			break;
		}
		
		byte = (byte & 0xFE) | bit; 
		
		fseek(file, -1, SEEK_CUR); 
		fwrite(&byte, 1, 1, file); 
		
		positions[i] = pos; 
		fprintf(pos_file, "%d\n", pos); 
	}
	
	fclose(file);
	fclose(pos_file);
	
	printf("[+] Message hidden successfully! Positions saved in pos.txt\n");
	
	free(positions);
}

int main(int argc, char *argv[]) {
	if (argc != 4) {
		fprintf(stderr, "Usage: %s <file> <cursor position> <message>\n", argv[0]);
		return 1;
	}
	
	const char *filename = argv[1];
	int start_pos = atoi(argv[2]);
	const char *message = argv[3];
	
	hide(filename, start_pos, message);
	return 0;
}
