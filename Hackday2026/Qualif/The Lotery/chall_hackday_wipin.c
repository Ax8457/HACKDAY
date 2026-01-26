#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <SDL2/SDL.h>
#include <math.h>
#include <emscripten/emscripten.h>

#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"    

#define WIDTH 800
#define HEIGHT 800
#define MAX_SCORE 7235959     
	
/*
wasm : emcc chall_hackday_wipin.c chall.c   -s USE_SDL=2   -s USE_SDL_TTF=2   -O3   -s INITIAL_MEMORY=64MB   -s ALLOW_MEMORY_GROWTH=1   --preload-file rules.txt   --preload-file Ithaca.ttf   -o index.html -s MINIFY_HTML=0 -s ASSERTIONS=1 -s FORCE_FILESYSTEM=1 -s FULL_ES2=1
edit frozen cache /!\ 
 1828  vim.tiny sudo vim.tiny /usr/share/emscripten/tools/config_template.py 
 1829  sudo vim.tiny /usr/share/emscripten/tools/config_template.py 
 https://emscripten.org/docs/porting/connecting_cpp_and_javascript/Interacting-with-code.html#interacting-with-code-call-javascript-from-native
 https://www.usenix.org/conference/usenixsecurity20/presentation/lehmann
*/

//flag
void print_flag() {
    FILE *f = fopen("flag.txt", "r");
    if (!f) {
        perror("Failed to open flag.txt");
        return;
    }

    char buffer[256];
    while (fgets(buffer, sizeof(buffer), f)) {
        printf("%s", buffer);
    }

    fclose(f);
}

EM_JS(char*, get_username_js, (), {
    var name = prompt("Enter your username:");
    if (!name) name = "";
    var lengthBytes = lengthBytesUTF8(name) + 1;
    var stringOnWasmHeap = _malloc(lengthBytes);
    stringToUTF8(name, stringOnWasmHeap, lengthBytes);
    return stringOnWasmHeap;
});

//ext
extern unsigned char chall_png[];
extern unsigned int chall_png_len;

void die(const char *msg) {
    fprintf(stderr, "%s: %s\n", msg, SDL_GetError());
    exit(1);
}

uint32_t* load_pixels(int *width, int *height) {
    int channels;
    unsigned char *data = stbi_load_from_memory(chall_png, chall_png_len, width, height, &channels, 4);
    if (!data) {
    die("Error loading embedded PNG\n");
    return NULL;
    }

    size_t count = (size_t)(*width) * (*height);
    uint32_t *pixels = malloc(count * sizeof(uint32_t));
    if (!pixels) {
    die("Error allocating memory for pixels\n");
    stbi_image_free(data);
    return NULL;
    }

    for (size_t i = 0; i < count; i++) {
    unsigned char r = data[i * 4 + 0];
    unsigned char g = data[i * 4 + 1];
    unsigned char b = data[i * 4 + 2];
    unsigned char a = data[i * 4 + 3];
    pixels[i] = ((uint32_t)a << 24) | ((uint32_t)r << 16)
              | ((uint32_t)g << 8) | (uint32_t)b;
    }

    stbi_image_free(data);
    return pixels;
}

void generate_noise(uint32_t *pixels){
    for(int i = 0; i < WIDTH*HEIGHT; i++){
        pixels[i] = rand();
    }
}

void win(uint32_t *pixels, int counter){
    if (counter > 255) counter = 0;
    uint32_t color = (0xFF << 24) | (0x00 << 16) | (counter << 8) | 0x00;
    for(int i = 0; i < WIDTH*HEIGHT; i++){
    pixels[i] = color;
    } 
}


typedef struct {
    char username[64];
    int score;
} Player;

static int w, h;
static int counter_mov = 0;
static int cursor_pixel;
static int cx, cy;
static int radius;

static Uint32 startTime = 0;

static Uint32* flag_pixels = NULL;
static Uint32* pixels = NULL;
static SDL_Window *window = NULL;
static SDL_Renderer *renderer = NULL;
static SDL_Texture *texture = NULL;

static bool game_over = false;
static int win_counter = 0;


static Player player;

void game_loop(void) {
    if (game_over) return;

    SDL_UpdateTexture(texture, NULL, pixels, WIDTH * 4);
    SDL_RenderClear(renderer);
    SDL_RenderCopy(renderer, texture, NULL, NULL);
    SDL_RenderPresent(renderer);

    static int last_score = 0;

    SDL_Event event;
    while (SDL_PollEvent(&event)) {
        if (event.type == SDL_QUIT || 
           (event.type == SDL_KEYDOWN && event.key.keysym.sym == SDLK_ESCAPE)) {
            emscripten_cancel_main_loop();
            return;
        }

        if (event.type == SDL_MOUSEMOTION) {
            cursor_pixel = event.motion.x + event.motion.y * WIDTH;
            cx = cursor_pixel % WIDTH;
            cy = cursor_pixel / WIDTH;
            radius = 10;

            for(int y = cy - radius; y <= cy + radius; y++) {
                for(int x = cx - radius; x <= cx + radius; x++) {
                    if(x >= 0 && x < WIDTH && y >= 0 && y < HEIGHT) {
                        int dx = x - cx;
                        int dy = y - cy;
                        if(dx*dx + dy*dy <= radius*radius) {
                            int idx = y * WIDTH + x;
                            if(pixels[idx] != flag_pixels[idx]) {
                                pixels[idx] = flag_pixels[idx];
                                counter_mov += 1;
                                player.score += 5;  
                            }
                        }
                    }
                }
            }
        }
    }

    
    if (player.score != last_score) {
        printf("Score: %d\n", player.score);
        last_score = player.score;
    }


    if (player.score == MAX_SCORE && !game_over) {
        printf("You won, %s!\n", player.username);
        print_flag();
        game_over = true; 
    }
}

int main(int argc, char *argv[]) { 
    printf("[***] Starting program...\n");

    char *js_name = get_username_js();
    player.score = 0;
    if (js_name) {
	strcpy(player.username, js_name);
    }
    

    printf("Starting program for: %s\n", player.username);

    printf("[INFO] Wipe the screen to start the game\n");

    flag_pixels = load_pixels(&w, &h);
    if (!flag_pixels)
        die("Failed to load PNG");

    if (SDL_Init(SDL_INIT_VIDEO) < 0)
        die("SDL init failed");

    atexit(SDL_Quit);

    window = SDL_CreateWindow(
        "ESIEE Paris - HACKDAY 2026 - Wipe Wipe Wipe !!", 0, 50, WIDTH, HEIGHT, 0
    );    
    if (!window)
        die("SDL_CreateWindow failed");

    renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_PRESENTVSYNC);
    if (!renderer)
        die("SDL_CreateRenderer failed");

    texture = SDL_CreateTexture(
        renderer, SDL_PIXELFORMAT_XRGB8888,
        SDL_TEXTUREACCESS_STREAMING,
        WIDTH, HEIGHT
    );
    if (!texture)
        die("SDL_CreateTexture failed");

    pixels = calloc(WIDTH * HEIGHT, sizeof(Uint32));
    if (!pixels)
        die("malloc failed");

    emscripten_set_main_loop(game_loop, 0, 1);

    return 0;
}

