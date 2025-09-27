
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
extern void initialize_client(char *);

// Params: Secret Seed
// Returns: Signer Handle
extern int initialize_signer(char *);

// Params: Signer Handle, Data (as string), App Id
// Returns: Submitted Transaction Handle
extern int do_submit_data(int, char *, int);

// Params: Submitted Transaction Handle
// Returns: Transaction Receipt Handle
extern int get_transaction_receipt(int);

// Params: Transaction Receipt Handle
// Returns: Rust Allocated Receipt struct
// Note: Make sure to call receipt_free!
extern Receipt *receipt_new(int);

// Params: Rust Allocated Receipt struct
extern void receipt_free(Receipt *);

char *hash_to_hex(const unsigned char *hash, size_t len)
{
    size_t hex_len = len * 2;
    char *hex = malloc(hex_len + 1);
    if (!hex)
    {
        return NULL;
    }

    for (size_t i = 0; i < len; ++i)
    {
        sprintf(hex + (i * 2), "%02x", hash[i]);
    }
    hex[hex_len] = '\0';
    return hex;
}

int main(void)
{
    // Avail Client
    initialize_client("https://turing-rpc.avail.so/rpc");

    // Alice
    int signer_handle = initialize_signer("bottom drive obey lake curtain smoke basket hold race lonely fit walk");

    // Submit Data
    int submitted_tx_handle = do_submit_data(signer_handle, "Hello From C", 2);

    // Wait for TX to be included
    int receipt_handle = get_transaction_receipt(submitted_tx_handle);

    Receipt *receipt = receipt_new(receipt_handle);
    char *block_hash = hash_to_hex(receipt->block_hash, 32);
    char *tx_hash = hash_to_hex(receipt->transaction_hash, 32);

    printf("C: Block Height: %d, Tx Index: %d\n", receipt->block_height, receipt->transaction_index);
    printf("C: Block Hash: 0x%s, Tx Hash: 0x%s\n", block_hash, tx_hash);
    receipt_free(receipt);
    free(block_hash);
    free(tx_hash);

    return 0;
}