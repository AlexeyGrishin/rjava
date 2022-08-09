import io.github.rvm.RVM;

public class TailRecursion {
    public static void main(String[] args) {
        int start1 = RVM.tick();
        int fib1 = tailCallFibonacci(40);
        int end1 = RVM.tick();
        RVM.print("Fibonacci without optimization = " + fib1 + " (took " + (end1 - start1) + "ms)\n");

        int start2 = RVM.tick();
        int fib2 = tailCallFibonacciOptimized(40);
        int end2 = RVM.tick();
        RVM.print("Fibonacci with optimization = " + fib2 + " (took " + (end2 - start2) + "ms)\n");

    }

    public static int tailCallFibonacci(int nth) {
        return tailCallFibonacci(0, 1, nth);
    }

    private static int tailCallFibonacci(int prevPrevFib, int prevFib, int remaining) {
        if (remaining <= 0) return prevPrevFib;
        if (remaining == 1) {
            RVM.logState();
            return prevFib;
        }
        return tailCallFibonacci(prevFib, prevPrevFib + prevFib, remaining-1);
    }


    public static int tailCallFibonacciOptimized(int nth) {
        return tailCallFibonacciOptimized(0, 1, nth);
    }

    @RVM.TailRecursion
    private static int tailCallFibonacciOptimized(int prevPrevFib, int prevFib, int remaining) {
        if (remaining <= 0) return prevPrevFib;
        if (remaining == 1) {
            RVM.logState();
            return prevFib;
        }
        return tailCallFibonacciOptimized(prevFib, prevPrevFib + prevFib, remaining-1);
    }
}
