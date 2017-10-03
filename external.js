mergeInto(LibraryManager.library, {
    print_val: function(ch) {
      result+=String.fromCharCode(ch)
      document.getElementById("results").innerText=result
      
    },
    read_val : function (){
        return prompt("Char", "A")[0];       
    }
  });