

let setup = ()=>{
  z = 22;
  
  let a = ()=>{
    b()
  }

  return a
}

let v = setup()



let b = ()=>{
  console.log("b", z)
}


v()