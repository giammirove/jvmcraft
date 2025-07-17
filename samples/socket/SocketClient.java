
// Demonstrating Client-side Programming
import java.io.*;
import java.net.*;

public class SocketClient {

  // Initialize socket and input/output streams
  private Socket s = null;
  private DataInputStream in = null;
  private DataOutputStream out = null;

  // Constructor to put IP address and port
  public SocketClient(String addr, int port) {
    // Establish a connection
    try {
      s = new Socket(addr, port);
      System.out.println("Connected");

      // Takes input from terminal
      in = new DataInputStream(System.in);

      // Sends output to the socket
      out = new DataOutputStream(s.getOutputStream());
    } catch (UnknownHostException u) {
      System.out.println(u);
      return;
    } catch (IOException i) {
      System.out.println(i);
      return;
    }

    // String to read message from input
    String m = "";

    // Keep reading until "Over" is input
    int times = 0;
    while (!m.equals("Over")) {
      try {
        System.out.println("Type Something:");
        m = in.readLine();
        System.out.println("Sending " + m);
        System.out.println("Sending " + m.length());
        out.writeUTF(m);
        times += 1;
        System.out.println("=========================");
      } catch (IOException i) {
        System.out.println(i);
      }
    }

    // Close the connection
    try {
      in.close();
      out.close();
      s.close();
    } catch (IOException i) {
      System.out.println(i);
    }
  }

  public static void main(String[] args) {
    SocketClient c = new SocketClient("127.0.0.1", 5000);
  }
}
