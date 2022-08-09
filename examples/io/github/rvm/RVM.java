package io.github.rvm;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

public class RVM {

    native public static void print(String message);

    native public static void print(Object object);

    native public static void print(int value);

    native public static void println();

    native public static void logState();

    native public static int tick();

    native public static int heapSize();

    @Target(ElementType.METHOD)
    @Retention(RetentionPolicy.RUNTIME)
    public @interface TailRecursion {}

    @Target(ElementType.METHOD)
    @Retention(RetentionPolicy.RUNTIME)
    public @interface Mem {}

    @Target(ElementType.METHOD)
    @Retention(RetentionPolicy.RUNTIME)
    public @interface AutoFree {}

    public static Object getAnswer(MemEntry entry, Object[] arguments) {
        return entry == null ? null : entry.getAnswer(arguments);
    }

    public static Object getAnswer(MemEntry entry, Object arg1) {
        return entry == null ? null : entry.getAnswer(new Object[]{arg1});
    }

    public static Object getAnswer(MemEntry entry, Object arg1, Object arg2) {
        return entry == null ? null : entry.getAnswer(new Object[]{arg1, arg2});
    }


}
