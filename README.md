# Rlox

Rlox is a rust implementation of the lox language, which is a simple and concise __dynamically typed script language__ designed by Robert Nystrom in his book _Crafting Interpreters_. This repository now contains only a tree-walker interpreter for lox.

__Note__. Rlox isn't a faithful implementation to the standard specified from the book. I did some little changes based on my own appetite, such as removing inheritance ~~(partly because of my laziness)~~  and adding support for arrays. But other parts are almost the same. 

---

To start up a repl, just type

```
rlox
```

To run a lox script, type

```
rlox [script]
```

If the interpreter happened to encounter some errors, hopefully it will point out the problem and cease executing.

---

Here is an overview of rlox.

```javascript
var a = 1;
var array = [32768, "halo", [4,2]];
var a = "Number " + a; // variable shadowing
for (var i = 0; i < len(array); i = i + 1) { // classic C-styled for loop
    array = array + array[i]; // print is embedded into the language
}
for e in array { // or you can use a for-in loop
    print typeof(e); // rlox has 7 types: number, string, boolean, function, class, instance, array
}
while true {
    break; // rlox supports basic control flow statements: break, continue, return
}
fun add(a, b) {
    return a + b;
}
fun get_callback(n) {
    var base = n;
    return |y| base + y; // rlox supports closure and rust-like lambda expressions.
    					 // or you can write |y| { return base + y; }
}
let closure = get_callback(43);
print closure(-1);

class Block {
    init(width, height) {
        this.width = width;
        this.height = height;
    }
    square() {
        return this.width * this.height;
    }
}

var block = Block();
block.inin(3,4);
print block.square();
```

