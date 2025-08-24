### How to run the system

1. Configure the client with correct configuration for each (change the constants in /smpc-client/src/actor/consts.rs).
2. `cargo run`: Run multiple instances of clients with different configuration (at least 2 required). The configuration variables are:
   a. port: Specify the port number you want your client to run. This port will serve both http and websocket requests.
   b. private_input: The input which you don't want to reveal.
   c. random_value: Any random value in u64 which will be a substitute for the private_input. For the first client that connects to the server, thsi value won't be used but calculated in order to preserve the relation between private_inputs and random_values.
3. `cargo run`: Run the server (port 8080 by default) - which will run the Server actor that manages clients and acts as a relayer for message passing.
4. Make a request to the client one by one (at endpoint /) which in turn will call the server's endpoint /register-client which will store the client's url. The first request to this endpoint will trigger a timer for n seconds, after which server will stop accepting any new register-clients and will start the protocol.
5. From here, everything happens automatically. The server will try to establish the websocket connection with a handshake to each client and store the sinks and sources to these channels. If successful, the first client is sent _Initialize_ message. The first client then generates the Paillier cryptosystem keypair, encrypts its private input with public key and sends it to the server by wrapping the result with _Unicast_ message type specifying the destination to the next client id. The server unwraps the Unicast message, sees the destination and forwards it to the correct websocket sink. The first round continues till the last client (FirstRoundResponse message type) and the second round starts from the last client itself moving back up to the first client(SecondRoundResponse message type). As the last step, the first client decrypts the received message from the server and finally receives its public random output.
   Exact protocol is discussed [here](#protocol)

### MESSAGES

#### Server receiving

    1. Unicast<T>: Unwraps the message, sees the destination, relays the serialized message to the client without ever looking into the message
    2. Broadcast<T>: Unwraps the message, forwards the message to all the websocket sinks except the sender itself. (TODO)
    3. ResultResponse: Expected from the first client which indicates the protocol is complete and allows the server to close all the websocket connection and stop the Server Actor. Same server could be used for the next SMPC (TODO)

#### Client receiving

    1. Initialize: The first client receives this message, generates Paillier keypair, encrypts its private number/message. Builds the FirstRoundResponse, serializes the message and wraps the result in Unicast and sends it to the server by specifying the id+1 as the destination.
    2. FirstRoundResponse: All other clients receive this message in the first round where they raise the received value to the power of their private message. If this is the last client, it builds the SecondRoundResponse and sends it back to the server by specifying itself as the destination. Else, it builds the FirstRoundResponse by specifying id+1 as the destination and wraps it in Unicast and sends back to server.
    3. SecondRoundResponse: All the clients receive this message in the second round where they encrypt their random value with the first client's pubkey (included in the message), calculate its mod inverse and mod multiplies with the received value. Sends SecondRoundResponse back to the id-1 by wrapping it in the Unicast. If the client is the first client, it sends back ResultResponse back to the server which indicates that the protocol is complete.

### Server acts as a storage at first and after the protocol begins it acts only as a relayer

    Secure Multiparty computation is usually a decentralized process with no need of the server. While adding a server makes the process easier by recording the total clients and counting and maintaing websocket connnections, it is completely redundant. We can make the protocol completely decentralized by making the clients more intelligent about their neighbors (like a doubly linked list) - TODO

### PROTOCOL <a name="headin"></a>

Paillier cryptosystem is an asymmetric public key cryptosystem whose hardness depends on calculating n_th residue class. The algorithm is discussed in more detail [here](https://en.wikipedia.org/wiki/Paillier_cryptosystem) but to visualize given a message in a multiplicative group of n = p\*q(two large primes), the encryption hides the message in a bigger mulitplicative group of n^2 with some randomness (hence asymmetric). If the generator was chosen wisely (usually choosing g=n+1 ensures proper decryption), the decryption process will bring that number back to the original number in Zn.

The most important property of the Paillier cryptosystem is: additive homomorphic property. where, multiplying the encryption of two messages encrypted with same pub key results in the encryption of sum of messages.
E(m1)\*E(m2) = E(m1+m2)

#### Secure MultiParty Multiplication protocol

Suppose E is an additive encryption scheme everyone has agreed upon (Paillier Cryptosystem). The
1st user has a key pair based on this encryption scheme given by
e(encryption key) which everyone knows and d(decryption key) only they
know.\

The first user starts the first round by passing its encrypted secret
input, the subsequent users raise the received value to the power of
their secret inputs.

- The first user encrypts its input $x_{1}$, $E(x_{1},e)$, and sends it
  to user.

- First round: For $i=2$ to $n$:

  - The $i^{th}$ user raises the power of the received value to its input
    $x_{i}$, $E(x_{1},e)^{x_{2}\cdots x_i}$, and sends it to
    ${i+1}^{th}$ user.

  When the encrypted value reaches the last user, we start the second
  round where each user takes a random value, encrypts it and raises its
  inverse to the power of the received value, and finally the first
  element gets its $r_1$ which satisfies the above relation.

- For $i=n$ to $2$:

  - The $i_{th}$ user randomly selects its private output $r_{i}$,
    encrypts it, $E(r_{i},e)$, computes its inverse, $E(r_{i},e)^{-1}$,
    multiplies the received value by that,
    $E(x_{1},e)^{x_{2}\cdots x_i} * \prod\limits_{j=i}^{n}E(r_{j},e)^{-1}$,
    and sends it back to the ${i-1}^{th}$ user.

- The first $1^{st}$ user decrypts the received value from $2^{nd}$ user
  and sets it as its output share $r_{1}$.
  $$r_{1}=D\left(E(x_{1},e)^{x_{2}\cdots x_i} * \left(\prod\limits_{i=2}^{n} E(x_{i},e)^{-1}\right),d\right)$$

##### How does this give the relation that we want?

Due to the homomorphic property of E, we have,
$$\sum\limits_{i=1}^{n}r_{i} = D\left(E\left(\sum\limits_{i=1}^{n}r_{i},e\right),d\right)=D\left(\prod\limits_{i=1}^{n}E\left(r_{i},e\right),d\right)$$
$$= D\left(E(r_{1},e) * \prod\limits_{i=2}^{n}E(r_{i},e),d\right)$$
$$= D\left(E(x_{1},e)^{x_{2}\cdots x_n} * \prod\limits_{i=2}^{n}E(r_{i},e)^{-1} * \prod\limits_{i=2}^{n}E(r_{i},e),d\right)$$
$$= D\left(E(x_{1},e)^{x_{2}\cdots x_n},d\right)$$

$$
= D\left(E\left(\prod\limits_{i=1}^{n}x_{i},e\right),d\right)=\prod\limits_{i=1}^{n}x_i
$$

### Application

In many fields where only an average is to be known to the public, sharing each individual number/information could be a privacy risk, after performing the SMPC, we can share the random numbers generated that when multiplied give the same result as the addition of the private inputs. These random numbers carry no information and can be shared freely.
