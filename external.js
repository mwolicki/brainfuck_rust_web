mergeInto(LibraryManager.library, {
    read_val : function (){
        var r = prompt("Char", "A")
        if (r != null) {
            return r.charCodeAt(0)
        }
        else {
            return 0
        }
    }
  });