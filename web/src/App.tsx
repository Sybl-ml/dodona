import React, {Component} from 'react';
import './App.css';


class App extends Component {

  state = {
    contacts: [],
  }

  componentDidMount() {
    fetch("http://localhost:3001/api")
    .then(res => res.json())
    .then((data) => {
      this.setState({contacts: data})
    })
    .catch(console.log)
  }

  public render() {
    return (
      <div className="App">
      <header className="App-header">
        <img src="logo.png" className="App-logo" alt="logo" />
        <h1>
          Welcome to Sybl
        </h1>
        <h3>Distributed ML with Ensemble Methods</h3>
        <p>
          
        </p>
      </header>
    </div>
    );
  }
}

export default App;