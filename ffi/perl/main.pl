#!/usr/bin/perl
use FFI::Platypus 2.00;
use FFI::CheckLib qw( find_lib_or_die );
use File::Basename qw( dirname );
use FFI::Platypus::Buffer qw( buffer_to_scalar );

my $ffi = FFI::Platypus->new( api => 2, lang => 'Rust' );
$ffi->lib(
    find_lib_or_die(
        lib        => 'avail_rust_ffi',
        libpath    => './../target/debug',
        systempath => [],
    )
);

$ffi->attach( initialize_client => ['string'] => 'isize' );

# Params: Secret Seed
# Returns: Signer Handle
$ffi->attach( initialize_signer => ['string'] => 'isize' );

# // Params: Signer Handle, Data (as string), App Id
# // Returns: Submitted Transaction Handle
$ffi->attach( do_submit_data => ['isize', 'string', 'isize'] => 'isize' );

# // Params: Submitted Transaction Handle
# // Returns: Transaction Receipt Handle
$ffi->attach( get_transaction_receipt => ['isize'] => 'isize' );

# Params: Transaction Receipt Handle
# Returns: Rust Allocated Receipt struct
# Note: Make sure to call receipt_free!
$ffi->attach( receipt_new => ['isize'] => 'opaque' );

# Params: Rust Allocated Receipt struct
$ffi->attach( receipt_free => ['opaque'] => [] );

# Getters because I am too dumb to figure out how to pass a pointer
# and access member fields of that pointer
$ffi->attach( receipt_block_height => ['opaque'] => 'isize' );
$ffi->attach( receipt_block_hash => ['opaque'] => 'opaque' );
$ffi->attach( receipt_transaction_hash => ['opaque'] => 'opaque' );
$ffi->attach( receipt_transaction_index => ['opaque'] => 'isize' );

initialize_client("https://turing-rpc.avail.so/rpc");
my $signer_handle = initialize_signer("bottom drive obey lake curtain smoke basket hold race lonely fit walk");
my $submitted_tx_handle = do_submit_data($signer_handle, "Hello From Perl", 2);
my $receipt_handle = get_transaction_receipt($submitted_tx_handle);
my $receipt = receipt_new($receipt_handle);
my $block_height = receipt_block_height($receipt);
my $transaction_index = receipt_transaction_index($receipt);
print "Perl: Block Height: ", $block_height, ", Transaction Index: ", $transaction_index, "\n";

my $hash_ptr = receipt_block_hash($receipt);
my $hash_raw = buffer_to_scalar($hash_ptr, 32);
my $hash = "0x" . unpack('H*', $hash_raw);
print "Perl: Block Hash: ", $hash, "\n";

my $hash_ptr = receipt_transaction_hash($receipt);
my $hash_raw = buffer_to_scalar($hash_ptr, 32);
my $hash = "0x" . unpack('H*', $hash_raw);
print "Perl: Transaction Hash: ", $hash, "\n";

receipt_free($receipt) if $receipt;
