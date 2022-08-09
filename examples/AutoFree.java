import io.github.rvm.RVM;

public class AutoFree {

    public static void main(String[] args) {
        RVM.logState();
        int heap1 = RVM.heapSize();
        countAllocate(100);
        RVM.logState();
        int heap2 = RVM.heapSize();

        countAllocateAutoFree(100);
        RVM.logState();
        int heap3 = RVM.heapSize();

        RVM.print("Allocated without auto-free " + (heap2 - heap1) + " values, with auto-free - " + (heap3 - heap2) + " values");
    }

    public static class Vector<T> {
        private Object[] items;
        private int idx = 0;

        public Vector(int capacity) {
            items = new Object[capacity];
        }

        public void add(T value) {
            items[idx++] = value;
        }

        public T get(int idx) {
            return (T)items[idx];
        }

        public int size() {
            return idx;
        }

    }

    public static int countAllocate(int count) {
        Vector<String> vector = new Vector<>(count);
        for (int i = 0; i < count; i++) {
            vector.add(i + "");
        }
        return vector.size();
    }


    @RVM.AutoFree
    public static int countAllocateAutoFree(int count) {
        Vector<String> vector = new Vector<>(count);
        for (int i = 0; i < count; i++) {
            vector.add(i + "");
        }
        return vector.size();
    }

}
