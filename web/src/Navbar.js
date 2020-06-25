import React from "react"
import { Navbar, Nav, NavItem, NavDropdown, Form, FormControl, Button } from 'react-bootstrap';

export class Header extends React.Component {
    render() {
        return (
		<Navbar bg="dark" variant="dark" fixed="top">
    			<Navbar.Brand href="#home">
      				<img
        			 alt=""
        			 src="/logo.png"
        			 height="30"
				 width="30"
        			 className="d-inline-block align-top"
				/>{' '}
      				<b>Sybl</b>
    			</Navbar.Brand>
			<Nav className="mr-auto">
      				<Nav.Link href="#home">Home</Nav.Link>
      				<Nav.Link href="#features">Features</Nav.Link>
      				<Nav.Link href="#pricing">Pricing</Nav.Link>
    		</Nav>
    		<Navbar.Collapse className="justify-content-end">
				<Navbar.Text>
					Signed in as: <a href="/login.html">John Doe</a>
				</Navbar.Text>
			</Navbar.Collapse>
  		</Navbar>
        );
    }
}

export default Header;