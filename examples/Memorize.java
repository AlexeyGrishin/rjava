import io.github.rvm.RVM;

public class Memorize {
    public static void main(String[] args) {
        Memorize mem = new Memorize();
        int start1 = RVM.tick();
        int fib1 = mem.naiveFibonacci(30);
        int end1 = RVM.tick();
        RVM.print("Fibonacci without memoization = " + fib1 + " (took " + (end1 - start1) + "ms)\n");

        int start2 = RVM.tick();
        int fib2 = mem.naiveFibonacciMem(30);
        int end2 = RVM.tick();
        RVM.print("Fibonacci with memoization = " + fib2 + " (took " + (end2 - start2) + "ms)\n");
        RVM.logState();
    }


    public int naiveFibonacci(int nth) {
        if (nth <= 0) return 0;
        if (nth <= 2) return 1;
        return naiveFibonacci(nth - 1) + naiveFibonacci(nth - 2);
    }


    @RVM.Mem
    public Integer naiveFibonacciMem(Integer nth) {
        if (nth <= 0) return 0;
        if (nth <= 2) return 1;
        return naiveFibonacciMem(nth - 1) + naiveFibonacciMem(nth - 2);
    }
}
