#include <stdio.h>
#include <stdlib.h>
typedef struct
{
    int block_height;
    unsigned char block_hash[32];
    int transaction_index;
    unsigned char transaction_hash[32];
    int handle;
} Receipt;

extern void hello_from_rust();
extern void initialize_client(const char *);

// Params: Secret Seed
// Returns: Signer Handle
extern int initialize_signer(const char *);

// Params: Signer Handle, Data (as string), App Id
// Returns: Submitted Transaction Handle
extern int do_submit_data(int, const char *, int);

// Params: Submitted Transaction Handle
// Returns: Transaction Receipt Handle
extern int get_transaction_receipt(int);

// Params: Transaction Receipt Handle
// Returns: Rust Allocated Receipt struct
// Note: Make sure to call receipt_free!
extern Receipt *receipt_new(int);

// Params: Rust Allocated Receipt struct
extern void receipt_free(Receipt *);
