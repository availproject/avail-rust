#include <stdio.h>
#include <stdlib.h>
#include "main.h"

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