enum ServerMessageType{
    Initialize,
    Forward,
    Reverse,
    Result,
} 

struct Message<T>{
    message_type: ServerMessageType,
    data: T,
    to: String,
}

