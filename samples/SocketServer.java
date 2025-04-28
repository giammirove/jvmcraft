
// Demonstrating Server-side Programming
import java.net.*;
import java.io.*;

public class SocketServer {

  // Initialize socket and input stream
  private Socket s = null;
  private ServerSocket ss = null;
  private DataInputStream in = null;

  // Constructor with port
  public SocketServer(int port) {

    // Starts server and waits for a connection
    try {
      ss = new ServerSocket(port);
      System.out.println("Server started");

      System.out.println("Waiting for a client ...");

      s = ss.accept();
      System.out.println("Client accepted");

      // Takes input from the client socket
      in = new DataInputStream(
          new BufferedInputStream(s.getInputStream()));

      String m = "";

      // Reads message from client until "Over" is sent
      while (!m.equals("Over")) {
        try {
          m = in.readUTF();
          System.out.println(m);

        } catch (IOException i) {
          System.out.println(i);
        }
      }
      System.out.println("Closing connection");

      // Close connection
      s.close();
      in.close();
    } catch (IOException i) {
      System.out.println(i);
    }
  }

  public static void main(String args[]) {
    SocketServer s = new SocketServer(5000);
  }
}
