package io.github.rvm;

public class MemEntry {
    public final Object[] arguments;
    public final Object answer;

    public MemEntry next = null;

    public MemEntry(Object[] arguments, Object answer) {
        this.arguments = arguments;
        this.answer = answer;
    }

    public Object getAnswer(Object[] arguments) {
        if (arguments.length != this.arguments.length) return null;
        for (int i = 0; i < arguments.length; i++) {
            if (!arguments[i].equals(this.arguments[i])) {
                return next == null ? null : next.getAnswer(arguments);
            }
        }
        return answer;
    }


}
