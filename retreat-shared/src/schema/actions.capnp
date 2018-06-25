@0x8d5d0349107e2140;

enum Direction {
  forward @0;
  backward @1;
  left @2;
  right @3;
}

struct Action {
  union {
    move @0 :Direction;
    shoot @1 :Direction;
    jump @2 :Void;
  }
}

struct ClientActions {
    frame @0 :UInt8;
    actions @1 :List(Action);
}