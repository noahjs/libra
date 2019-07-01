# Libra client JSON API

All POST endpoints accept raw JSON.

## Methods

### `POST /create_wallet`
#### Request
No request body.
#### Response
```javascript
{
    "mnemonic": ".."
}
```

### `POST /create_wallet_account`
#### Request
```javascript
{
    "mnemonic": "..",
    "child_number": 1
}
```
#### Response
```javascript
{
    "child_number": 1,
    "address": "..",
    "private_key": ".."
}
```

### `POST /mint_coins`
#### Request
```javascript
{    
    "receiver": "..",
    "num_coins": 123
}
```
#### Response
```javascript
{
    "success": true
}
```

### `POST /transfer_coins`
#### Request
```javascript
{
    "sender_addr": "..",
    "receiver_addr": "..",
    "num_coins": 123,
    "gas_unit_price": 123, // optional
    "max_gas_amount": 123, // optional
    
    // either private key
    "private_key": "..",
    
    // or mnemonic and child number
    "mnemonic": "..",
    "child_number": 0
}
```
#### Response
```javascript
{
    "child_number": 1,
    "address": "..",
    "private_key": ".."
}
```

### `GET /get_latest_account_state/<addr>`
#### Parameters
`<addr>` (path segment)  - Account address in hexadecimal form.
#### Response
```javascript
{
    "balance": 100,
    "sequence_number": 1,
    "authentication_key": "..",
    "sent_events_count": 1,
    "received_events_count": 0
}
```

### `GET /get_committed_txn_by_acc_seq/<addr>?<sequence_number>&<fetch_events>`
#### Parameters
- `<addr>` (path segment)  - Account address in hexadecimal form.
- `<sequence_number>` - The sequence number of the transaction.
 -`<fetch_events>`

### `GET /get_committed_txn_by_range?<start_version>&<limit>&<fetch_events>`
#### Parameters
- `<start_version>`
- `<limit>`
- `<fetch_events>`

### `GET /get_events_by_account_and_type/<addr>?<event_type>&<start_seq_number>&<limit>&<ascending>`
#### Parameters
- `<addr>` (path segment) - Account address in hexadecimal form.
- `<get_events_by_account_and_type>` - `sent` or `received` 
- `<start_seq_number>`
- `<limit>`
- `<ascending>`
