mergeInto(LibraryManager.library, {
    read_val : function (current_output){
        var current_output = Module.UTF8ToString(current_output);
        results.innerText = current_output
        forceRedraw(results)
        var r = prompt("Char", "A")
        if (r != null) {
            return r.charCodeAt(0)
        }
        else {
            return 0
        }
    }
  });