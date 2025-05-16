export class History<T> {
  public history: T[] = [];
  private currentIndex: number = -1;

  push(item: T): void {
    if (this.history[this.currentIndex] == item) return;

    // If not at the end, remove forward history
    if (this.currentIndex < this.history.length - 1) {
      this.history = this.history.slice(0, this.currentIndex + 1);
    }

    this.history.push(item);
    this.currentIndex++;
  }

  back(): T | null {
    if (this.canGoBack()) {
      this.currentIndex--;
      return this.history[this.currentIndex];
    }
    return null;
  }

  forward(): T | null {
    if (this.canGoForward()) {
      this.currentIndex++;
      return this.history[this.currentIndex];
    }
    return null;
  }

  current(): T | null {
    return this.currentIndex >= 0 ? this.history[this.currentIndex] : null;
  }

  canGoBack(): boolean {
    return this.currentIndex > 0;
  }

  canGoForward(): boolean {
    return this.currentIndex < this.history.length - 1;
  }
}
