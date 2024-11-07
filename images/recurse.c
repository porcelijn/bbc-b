char b[16] = {0};
int f(int a)
{
  b[a] = a;
  if( a > 0)
    return f(--a);
  return 0;
}

int main() {
  return f(15);
}
