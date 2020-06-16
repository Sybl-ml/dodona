import React, {Component} from 'react';
import './App.css';
// import List from './List';

class App extends Component {

  state = {
    contacts: [],
    apiMessage: "",
  }

  componentDidMount() {
    fetch("http://localhost:3001/api/hello")
    .then(res => res.text())
      .then(
        (result) => {
          this.setState({
            apiMessage: result
          });
        },
        (error) => {
          console.log("Error");
        }
      )
    fetch("http://localhost:3001/api")
    .then(res => res.json())
      .then(
        (result) => {
          console.log(result.name);
          var joined = this.state.contacts.concat(result);
          this.setState({
            isLoaded: true,
            contacts: joined
          });
        },
        (error) => {
          this.setState({
            isLoaded: true,
            contacts: []
          });
        }
      )
  }

  render() {
    const { apiMessage, contacts } = this.state;
    return (
    <div className="App">
      <header className="App-header">
        <img src="logo.png" className="App-logo" alt="logo" />
        <h1>
          Welcome to Sybl
        </h1>
        <h3>Distributed ML with Ensemble Methods</h3>
        <h3>Data from API</h3>
        <p>
          {apiMessage}
        </p>
        <ul>
            {contacts.map((contact, index) => (
                <li key={index}>
                    {contact.name} {contact.age}
                </li>
            ))}
            {console.log(contacts)}
        </ul>
      </header>
    </div>
    );
  }
}

export default App;
