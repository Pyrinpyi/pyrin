

```bash

jq "to_entries | .[500]" blocks_transactions.json


```

```json
{
  "key": "7826f4ca01ed52a998d29f9df83deaab866da344ddc3593288098490091f6cb9",
  "value": [
    {
      "version": 0,
      "inputs": [],
      "outputs": [
        {
          "value": 50000000000,
          "scriptPublicKey": "000020600171a6c9e34684d67c4a6a28f695466bf5b2543c63763193351c08eabfb903ac"
        }
      ],
      "lockTime": 0,
      "subnetworkId": "0100000000000000000000000000000000000000",
      "gas": 0,
      "payload": "506400000000000000743ba40b00000000002220600171a6c9e34684d67c4a6a28f695466bf5b2543c63763193351c08eabfb903ac302e31342e352f707972696e6d696e65722d302e31342e35",
      "mass": 0,
      "id": "3d2108340713b29f2010eb55a5421792748074b26e2cb1a2c490ef4f905b211d"
    }
  ]
}                                                                                                                     
```

with --to-address

```json
{
  "key": "3018421fedc2ac22dae43fc796a8b5d7029b34699dbe4ad8123e0f54f1b63cc2",
  "value": [
    {
      "version": 0,
      "inputs": [],
      "outputs": [
        {
          "value": 50000000000,
          "scriptPublicKey": "000020600171a6c9e34684d67c4a6a28f695466bf5b2543c63763193351c08eabfb903ac",
          "address": "pyrin:qpsqzudxe835dpxk039x528kj4rxhadj2s7xxa33jv63cz82h7usxzqcvna3e"
        }
      ],
      "lockTime": 0,
      "subnetworkId": "0100000000000000000000000000000000000000",
      "gas": 0,
      "payload": "264b00000000000000743ba40b00000000002220600171a6c9e34684d67c4a6a28f695466bf5b2543c63763193351c08eabfb903ac302e31342e352f707972696e6d696e65722d302e31342e35",
      "mass": 0,
      "id": "418267474dee8b58f385dacc90d08b1d1b75773b5f85d36fcdbd45b007527a29"
    }
  ]
}
```