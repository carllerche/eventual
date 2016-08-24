var searchIndex = {};
searchIndex['eventual'] = {"items":[[0,"","eventual","Composable primitives for asynchronous computations",null,null],[3,"Future","","",null,null],[3,"Complete","","An object that is used to fulfill or reject an associated Future.",null,null],[3,"Receipt","","",null,null],[3,"Stream","","",null,null],[3,"StreamIter","","",null,null],[3,"Sender","","The sending half of `Stream::pair()`. Can only be owned by a single task at\na time.",null,null],[3,"BusySender","","",null,null],[3,"Timer","","Provides timeouts as a `Future` and periodic ticks as a `Stream`.",null,null],[4,"AsyncError","","",null,null],[13,"Failed","","",0,null],[13,"Aborted","","",0,null],[5,"join","","",null,{"inputs":[{"name":"j"}],"output":{"name":"future"}}],[5,"background","","This method backgrounds a task onto a task runner waiting for complete to be called.\nCurrently we only support using a ThreadPool as the task runner itself.",null,{"inputs":[{"name":"r"},{"name":"box"}],"output":{"name":"future"}}],[5,"defer","","This method defers a task onto a task runner until we can complete that call.\nCurrently we only support using a ThreadPool as the task runner itself.",null,{"inputs":[{"name":"r"},{"name":"a"}],"output":{"name":"future"}}],[5,"select","","",null,{"inputs":[{"name":"s"}],"output":{"name":"future"}}],[5,"sequence","","Returns a `Stream` consisting of the completion of the supplied async\nvalues in the order that they are completed.",null,{"inputs":[{"name":"i"}],"output":{"name":"stream"}}],[11,"pair","","",1,null],[11,"of","","Returns a future that will immediately succeed with the supplied value.",1,{"inputs":[{"name":"future"},{"name":"t"}],"output":{"name":"future"}}],[11,"error","","Returns a future that will immediately fail with the supplied error.",1,{"inputs":[{"name":"future"},{"name":"e"}],"output":{"name":"future"}}],[11,"lazy","","Returns a future that won't kick off its async action until\na consumer registers interest.",1,{"inputs":[{"name":"future"},{"name":"f"}],"output":{"name":"future"}}],[11,"map","","",1,{"inputs":[{"name":"future"},{"name":"f"}],"output":{"name":"future"}}],[11,"map_err","","Returns a new future with an identical value as the original. If the\noriginal future fails, apply the given function on the error and use\nthe result as the error of the new future.",1,{"inputs":[{"name":"future"},{"name":"f"}],"output":{"name":"future"}}],[11,"spawn","","Returns a `Future` representing the completion of the given closure.\nThe closure will be executed on a newly spawned thread.",1,{"inputs":[{"name":"future"},{"name":"f"}],"output":{"name":"future"}}],[11,"to_stream","","An adapter that converts any future into a one-value stream",1,{"inputs":[{"name":"future"}],"output":{"name":"stream"}}],[11,"is_ready","","",1,{"inputs":[{"name":"future"}],"output":{"name":"bool"}}],[11,"is_err","","",1,{"inputs":[{"name":"future"}],"output":{"name":"bool"}}],[11,"poll","","",1,{"inputs":[{"name":"future"}],"output":{"name":"result"}}],[11,"ready","","",1,{"inputs":[{"name":"future"},{"name":"f"}],"output":{"name":"receipt"}}],[11,"await","","",1,{"inputs":[{"name":"future"}],"output":{"name":"asyncresult"}}],[11,"pair","","",1,null],[11,"fmt","","",1,{"inputs":[{"name":"future"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"drop","","",1,{"inputs":[{"name":"future"}],"output":null}],[11,"cancel","","",2,{"inputs":[{"name":"receipt"}],"output":{"name":"option"}}],[11,"complete","","Fulfill the associated promise with a value",3,{"inputs":[{"name":"complete"},{"name":"t"}],"output":null}],[11,"fail","","Reject the associated promise with an error. The error\nwill be wrapped in `Async::Error::Failed`.",3,{"inputs":[{"name":"complete"},{"name":"e"}],"output":null}],[11,"abort","","",3,{"inputs":[{"name":"complete"}],"output":null}],[11,"is_ready","","",3,{"inputs":[{"name":"complete"}],"output":{"name":"bool"}}],[11,"is_err","","",3,{"inputs":[{"name":"complete"}],"output":{"name":"bool"}}],[11,"ready","","",3,{"inputs":[{"name":"complete"},{"name":"f"}],"output":null}],[11,"await","","",3,{"inputs":[{"name":"complete"}],"output":{"name":"asyncresult"}}],[11,"is_ready","","",3,{"inputs":[{"name":"complete"}],"output":{"name":"bool"}}],[11,"is_err","","",3,{"inputs":[{"name":"complete"}],"output":{"name":"bool"}}],[11,"poll","","",3,{"inputs":[{"name":"complete"}],"output":{"name":"result"}}],[11,"ready","","",3,{"inputs":[{"name":"complete"},{"name":"f"}],"output":{"name":"receipt"}}],[11,"drop","","",3,{"inputs":[{"name":"complete"}],"output":null}],[11,"fmt","","",3,{"inputs":[{"name":"complete"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"cancel","","",2,{"inputs":[{"name":"receipt"}],"output":{"name":"option"}}],[11,"join","collections::vec","",4,{"inputs":[{"name":"vec"},{"name":"complete"}],"output":null}],[11,"pair","eventual","Creates a new `Stream`, returning it with the associated `Sender`.",5,null],[11,"empty","","Returns a Stream that will immediately succeed with the supplied value.",5,{"inputs":[{"name":"stream"}],"output":{"name":"stream"}}],[11,"collect","","Asyncronously collects the items from the `Stream`, returning them sorted by order of\narrival.",5,{"inputs":[{"name":"stream"}],"output":{"name":"future"}}],[11,"iter","","Synchronously iterate over the `Stream`",5,{"inputs":[{"name":"stream"}],"output":{"name":"streamiter"}}],[11,"each","","Sequentially yields each value to the supplied function. Returns a\nfuture representing the completion of the final yield.",5,{"inputs":[{"name":"stream"},{"name":"f"}],"output":{"name":"future"}}],[11,"filter","","Returns a new stream containing the values of the original stream that\nmatch the given predicate.",5,{"inputs":[{"name":"stream"},{"name":"f"}],"output":{"name":"stream"}}],[11,"map","","Returns a new stream representing the application of the specified\nfunction to each value of the original stream.",5,{"inputs":[{"name":"stream"},{"name":"f"}],"output":{"name":"stream"}}],[11,"map_async","","Returns a new stream representing the application of the specified\nfunction to each value of the original stream. Each iteration waits for\nthe async result of the mapping to realize before continuing on to the\nnext value.",5,{"inputs":[{"name":"stream"},{"name":"f"}],"output":{"name":"stream"}}],[11,"map_err","","Returns a new stream with an identical sequence of values as the\noriginal. If the original stream errors, apply the given function on\nthe error and use the result as the error of the new stream.",5,{"inputs":[{"name":"stream"},{"name":"f"}],"output":{"name":"stream"}}],[11,"process","","",5,{"inputs":[{"name":"stream"},{"name":"usize"},{"name":"f"}],"output":{"name":"stream"}}],[11,"reduce","","Aggregate all the values of the stream by applying the given function\nto each value and the result of the previous application. The first\niteration is seeded with the given initial value.",5,{"inputs":[{"name":"stream"},{"name":"u"},{"name":"f"}],"output":{"name":"future"}}],[11,"reduce_async","","Aggregate all the values of the stream by applying the given function\nto each value and the realized result of the previous application. The\nfirst iteration is seeded with the given initial value.",5,{"inputs":[{"name":"stream"},{"name":"x"},{"name":"f"}],"output":{"name":"future"}}],[11,"take","","Returns a stream representing the `n` first values of the original\nstream.",5,{"inputs":[{"name":"stream"},{"name":"u64"}],"output":{"name":"stream"}}],[11,"take_while","","",5,{"inputs":[{"name":"stream"},{"name":"f"}],"output":{"name":"stream"}}],[11,"take_until","","",5,{"inputs":[{"name":"stream"},{"name":"a"}],"output":{"name":"stream"}}],[11,"to_future","","",5,{"inputs":[{"name":"stream"}],"output":{"name":"future"}}],[11,"is_ready","","",5,{"inputs":[{"name":"stream"}],"output":{"name":"bool"}}],[11,"is_err","","",5,{"inputs":[{"name":"stream"}],"output":{"name":"bool"}}],[11,"poll","","",5,{"inputs":[{"name":"stream"}],"output":{"name":"result"}}],[11,"ready","","",5,{"inputs":[{"name":"stream"},{"name":"f"}],"output":{"name":"receipt"}}],[11,"await","","",5,{"inputs":[{"name":"stream"}],"output":{"name":"asyncresult"}}],[11,"pair","","",5,null],[11,"fmt","","",5,{"inputs":[{"name":"stream"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"drop","","",5,{"inputs":[{"name":"stream"}],"output":null}],[11,"cancel","","",2,{"inputs":[{"name":"receipt"}],"output":{"name":"option"}}],[11,"send","","Attempts to send a value to its `Stream`. Consumes self and returns a\nfuture representing the operation completing successfully and interest\nin the next value being expressed.",6,{"inputs":[{"name":"sender"},{"name":"t"}],"output":{"name":"busysender"}}],[11,"fail","","Terminated the stream with the given error.",6,{"inputs":[{"name":"sender"},{"name":"e"}],"output":null}],[11,"abort","","Fails the paired `Stream` with a cancellation error. This will\neventually go away when carllerche/syncbox#10 lands. It is currently\nneeded to keep the state correct (see async::sequence)",6,{"inputs":[{"name":"sender"}],"output":null}],[11,"send_all","","Send + 'static all the values in the given source",6,{"inputs":[{"name":"sender"},{"name":"s"}],"output":{"name":"future"}}],[11,"is_ready","","",6,{"inputs":[{"name":"sender"}],"output":{"name":"bool"}}],[11,"is_err","","",6,{"inputs":[{"name":"sender"}],"output":{"name":"bool"}}],[11,"poll","","",6,{"inputs":[{"name":"sender"}],"output":{"name":"result"}}],[11,"ready","","",6,{"inputs":[{"name":"sender"},{"name":"f"}],"output":{"name":"receipt"}}],[11,"drop","","",6,{"inputs":[{"name":"sender"}],"output":null}],[11,"is_ready","","",7,{"inputs":[{"name":"busysender"}],"output":{"name":"bool"}}],[11,"is_err","","",7,{"inputs":[{"name":"busysender"}],"output":{"name":"bool"}}],[11,"poll","","",7,{"inputs":[{"name":"busysender"}],"output":{"name":"result"}}],[11,"ready","","",7,{"inputs":[{"name":"busysender"},{"name":"f"}],"output":{"name":"receipt"}}],[11,"drop","","",7,{"inputs":[{"name":"busysender"}],"output":null}],[11,"fmt","","",6,{"inputs":[{"name":"sender"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"send_all","","",1,{"inputs":[{"name":"future"},{"name":"sender"}],"output":{"name":"future"}}],[11,"send_all","","",5,{"inputs":[{"name":"stream"},{"name":"sender"}],"output":{"name":"future"}}],[11,"cancel","","",2,{"inputs":[{"name":"receipt"}],"output":{"name":"option"}}],[11,"cancel","","",2,{"inputs":[{"name":"receipt"}],"output":{"name":"option"}}],[11,"next","","",8,{"inputs":[{"name":"streamiter"}],"output":{"name":"option"}}],[11,"drop","","",8,{"inputs":[{"name":"streamiter"}],"output":null}],[11,"new","","Creates a new timer backed by a thread pool with 5 threads.",9,{"inputs":[{"name":"timer"}],"output":{"name":"timer"}}],[11,"with_capacity","","Creates a new timer backed by a thread pool with `capacity` threads.",9,{"inputs":[{"name":"timer"},{"name":"u32"}],"output":{"name":"timer"}}],[11,"timeout_ms","","Returns a `Future` that will be completed in `ms` milliseconds",9,{"inputs":[{"name":"timer"},{"name":"u32"}],"output":{"name":"future"}}],[11,"interval_ms","","Return a `Stream` with values realized every `ms` milliseconds.",9,{"inputs":[{"name":"timer"},{"name":"u32"}],"output":{"name":"stream"}}],[11,"clone","","",9,{"inputs":[{"name":"timer"}],"output":{"name":"timer"}}],[6,"AsyncResult","","",null,null],[8,"Join","","",null,null],[10,"join","","",10,{"inputs":[{"name":"join"},{"name":"complete"}],"output":null}],[8,"Select","","",null,null],[10,"select","","",11,{"inputs":[{"name":"select"},{"name":"complete"}],"output":null}],[8,"Async","","A value representing an asynchronous computation",null,null],[16,"Value","","",12,null],[16,"Error","","",12,null],[16,"Cancel","","",12,null],[10,"is_ready","","Returns true if `expect` will succeed.",12,{"inputs":[{"name":"async"}],"output":{"name":"bool"}}],[10,"is_err","","Returns true if the async value is ready and has failed",12,{"inputs":[{"name":"async"}],"output":{"name":"bool"}}],[10,"poll","","Get the underlying value if present",12,{"inputs":[{"name":"async"}],"output":{"name":"result"}}],[11,"expect","","Get the underlying value if present, panic otherwise",12,{"inputs":[{"name":"async"}],"output":{"name":"asyncresult"}}],[10,"ready","","Invokes the given function when the Async instance is ready to be\nconsumed.",12,{"inputs":[{"name":"async"},{"name":"f"}],"output":{"name":"cancel"}}],[11,"receive","","Invoke the callback with the resolved `Async` result.",12,{"inputs":[{"name":"async"},{"name":"f"}],"output":null}],[11,"await","","Blocks the thread until the async value is complete and returns the\nresult.",12,{"inputs":[{"name":"async"}],"output":{"name":"asyncresult"}}],[11,"fire","","Trigger the computation without waiting for the result",12,{"inputs":[{"name":"async"}],"output":null}],[11,"and","","This method returns a future whose completion value depends on the\ncompletion value of the original future.",12,{"inputs":[{"name":"async"},{"name":"u"}],"output":{"name":"future"}}],[11,"and_then","","This method returns a future whose completion value depends on the\ncompletion value of the original future.",12,{"inputs":[{"name":"async"},{"name":"f"}],"output":{"name":"future"}}],[11,"or","","This method returns a future whose completion value depends on the\ncompletion value of the original future.",12,{"inputs":[{"name":"async"},{"name":"a"}],"output":{"name":"future"}}],[11,"or_else","","This method returns a future whose completion value depends on the\ncompletion value of the original future.",12,{"inputs":[{"name":"async"},{"name":"f"}],"output":{"name":"future"}}],[8,"Pair","","",null,null],[16,"Tx","","",13,null],[10,"pair","","",13,null],[8,"Cancel","","",null,null],[10,"cancel","","",14,{"inputs":[{"name":"cancel"}],"output":{"name":"option"}}],[11,"is_ready","core::result","",15,{"inputs":[{"name":"result"}],"output":{"name":"bool"}}],[11,"is_err","","",15,{"inputs":[{"name":"result"}],"output":{"name":"bool"}}],[11,"poll","","",15,{"inputs":[{"name":"result"}],"output":{"name":"result"}}],[11,"ready","","",15,{"inputs":[{"name":"result"},{"name":"f"}],"output":{"name":"option"}}],[11,"await","","",15,{"inputs":[{"name":"result"}],"output":{"name":"asyncresult"}}],[11,"cancel","core::option","",16,{"inputs":[{"name":"option"}],"output":{"name":"option"}}],[11,"eq","eventual","",0,{"inputs":[{"name":"asyncerror"},{"name":"asyncerror"}],"output":{"name":"bool"}}],[11,"ne","","",0,{"inputs":[{"name":"asyncerror"},{"name":"asyncerror"}],"output":{"name":"bool"}}],[11,"failed","","",0,{"inputs":[{"name":"asyncerror"},{"name":"e"}],"output":{"name":"asyncerror"}}],[11,"aborted","","",0,{"inputs":[{"name":"asyncerror"}],"output":{"name":"asyncerror"}}],[11,"is_aborted","","",0,{"inputs":[{"name":"asyncerror"}],"output":{"name":"bool"}}],[11,"is_failed","","",0,{"inputs":[{"name":"asyncerror"}],"output":{"name":"bool"}}],[11,"unwrap","","",0,{"inputs":[{"name":"asyncerror"}],"output":{"name":"e"}}],[11,"take","","",0,{"inputs":[{"name":"asyncerror"}],"output":{"name":"option"}}],[11,"description","","",0,{"inputs":[{"name":"asyncerror"}],"output":{"name":"str"}}],[11,"cause","","",0,{"inputs":[{"name":"asyncerror"}],"output":{"name":"option"}}],[11,"fmt","","",0,{"inputs":[{"name":"asyncerror"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"fmt","","",0,{"inputs":[{"name":"asyncerror"},{"name":"formatter"}],"output":{"name":"result"}}]],"paths":[[4,"AsyncError"],[3,"Future"],[3,"Receipt"],[3,"Complete"],[3,"Vec"],[3,"Stream"],[3,"Sender"],[3,"BusySender"],[3,"StreamIter"],[3,"Timer"],[8,"Join"],[8,"Select"],[8,"Async"],[8,"Pair"],[8,"Cancel"],[4,"Result"],[4,"Option"]]};
initSearch(searchIndex);
