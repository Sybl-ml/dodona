import React from "react"
import { Navbar, Nav, NavDropdown,} from 'react-bootstrap';
import styled from "styled-components";
import MemoLogo from '../icons/Logo.js';
import {PrimaryButton} from './Buttons';

import Toggle from "./Toggler"

const HeaderBar = styled(Navbar)`
	min-height: 4rem;
    background: linear-gradient(${({ theme }) => theme.body}, ${({ theme }) => theme.body} 90%, transparent 100%);
    transition: all 0.25s linear;
`;


const Square = styled.div`
	position: absolute;
	top: -5%;
	left: 45%;
	width: 10%;
	height: 10%;
	filter: drop-shadow(0 1px 2px 0 0 rgba(0,0,0,.5));
	background-color:${({ theme }) => theme.body};
	transform: rotate(45deg);
`;


const Header = ({theme, toggleTheme}) => {
        return (
		<HeaderBar sticky="top">
			<HeaderBar.Brand href="/home">
				<MemoLogo 
					theme={theme} 
				/> 
			</HeaderBar.Brand>
				
			<Nav>
				<Nav.Link href="#product">Product</Nav.Link>
				<Nav.Link href="#meet">Meet The Team</Nav.Link>
				<Nav.Link href="#pricing">Pricing</Nav.Link>
    		</Nav>
    		<HeaderBar.Collapse className="justify-content-end">
				<Toggle theme={theme} toggleTheme={toggleTheme} />
				<Nav>
					<Nav.Link href="/login">Sign In</Nav.Link>

				</Nav>
				<PrimaryButton variant="primary" href="/register">SIGN UP NOW</PrimaryButton>
			</HeaderBar.Collapse>
  		</HeaderBar>
        );
    };
export default Header;