


// console.log(Array.of(1,2,3))



Object.prototype.forEach = function(){
  for (let key in this) {
    console.log(key, this[key])
  }
}



({a:1,b:2,c:3})

